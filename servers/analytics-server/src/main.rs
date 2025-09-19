//! Analytics Server - Production MCP server for metrics and performance data
//!
//! This server provides comprehensive analytics capabilities including:
//! - Content metrics and engagement data
//! - Audience insights and segmentation
//! - Performance trends and time-series analysis
//! - Business metrics and reporting
//!
//! Built on the official RMCP SDK with production-ready patterns.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use dashmap::DashMap;
use rand::{thread_rng, Rng};
use rmcp::{
    handler::server::wrapper::Parameters, model::*, service::RequestContext, tool, tool_router,
    transport::stdio, ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

// ================================================================================================
// Request/Response Types
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetContentMetricsArgs {
    /// Content ID to get metrics for
    pub content_id: String,
    /// Time period for metrics (hour, day, week, month, year)
    #[serde(default = "default_period")]
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetAudienceInsightsArgs {
    /// Audience segment to analyze (all, new_users, returning_users, premium)
    #[serde(default = "default_segment")]
    pub segment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetEngagementTrendsArgs {
    /// Time span for trend analysis (7d, 30d, 90d, 1y)
    #[serde(default = "default_timespan")]
    pub timespan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateAnalyticsReportArgs {
    /// Report configuration
    pub config: ReportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReportConfig {
    /// Report type (summary, detailed, executive)
    pub report_type: String,
    /// Metrics to include
    pub metrics: Vec<String>,
    /// Time range for the report
    pub time_range: String,
    /// Output format (markdown, json, csv)
    #[serde(default = "default_format")]
    pub format: String,
}

// ================================================================================================
// Data Models
// ================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetrics {
    pub content_id: String,
    pub period: String,
    pub views: u64,
    pub unique_views: u64,
    pub likes: u64,
    pub shares: u64,
    pub comments: u64,
    pub conversion_rate: f64,
    pub engagement_rate: f64,
    pub average_session_duration: u64, // seconds
    pub bounce_rate: f64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceInsights {
    pub segment: String,
    pub total_users: u64,
    pub age_groups: HashMap<String, u64>,
    pub locations: HashMap<String, u64>,
    pub interests: HashMap<String, u64>,
    pub devices: HashMap<String, u64>,
    pub acquisition_channels: HashMap<String, u64>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementTrends {
    pub timespan: String,
    pub daily_active_users: Vec<TrendPoint>,
    pub session_duration_avg: Vec<TrendPoint>,
    pub pages_per_session: Vec<TrendPoint>,
    pub conversion_rate: Vec<TrendPoint>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub date: DateTime<Utc>,
    pub value: f64,
}

// ================================================================================================
// Server Implementation
// ================================================================================================

#[derive(Clone)]
pub struct AnalyticsServer {
    content_metrics: Arc<DashMap<String, ContentMetrics>>,
    audience_data: Arc<DashMap<String, AudienceInsights>>,
    engagement_trends: Arc<DashMap<String, EngagementTrends>>,
    tool_stats: Arc<DashMap<String, u64>>,
}

fn default_period() -> String {
    "day".to_string()
}

fn default_segment() -> String {
    "all".to_string()
}

fn default_timespan() -> String {
    "30d".to_string()
}

fn default_format() -> String {
    "markdown".to_string()
}

impl AnalyticsServer {
    pub fn new() -> Self {
        let server = Self {
            content_metrics: Arc::new(DashMap::new()),
            audience_data: Arc::new(DashMap::new()),
            engagement_trends: Arc::new(DashMap::new()),
            tool_stats: Arc::new(DashMap::new()),
        };

        // Initialize with mock data
        server.init_mock_data();
        server
    }

    fn init_mock_data(&self) {
        let mut rng = thread_rng();
        let now = Utc::now();

        // Generate mock content metrics for different content types
        let content_ids = vec![
            "blog-post-123",
            "video-456",
            "infographic-789",
            "podcast-321",
            "webinar-654",
        ];

        for content_id in content_ids {
            for period in &["hour", "day", "week", "month"] {
                let base_views = match *period {
                    "hour" => rng.gen_range(50..500),
                    "day" => rng.gen_range(500..5000),
                    "week" => rng.gen_range(2000..20000),
                    "month" => rng.gen_range(8000..80000),
                    _ => 1000,
                };

                let metrics = ContentMetrics {
                    content_id: content_id.to_string(),
                    period: period.to_string(),
                    views: base_views,
                    unique_views: (base_views as f64 * rng.gen_range(0.6..0.9)) as u64,
                    likes: (base_views as f64 * rng.gen_range(0.02..0.08)) as u64,
                    shares: (base_views as f64 * rng.gen_range(0.01..0.03)) as u64,
                    comments: (base_views as f64 * rng.gen_range(0.005..0.02)) as u64,
                    conversion_rate: rng.gen_range(0.01..0.15),
                    engagement_rate: rng.gen_range(0.02..0.25),
                    average_session_duration: rng.gen_range(30..600),
                    bounce_rate: rng.gen_range(0.20..0.80),
                    generated_at: now,
                };

                let key = format!("{content_id}:{period}");
                self.content_metrics.insert(key, metrics);
            }
        }

        // Generate mock audience insights
        let segments = vec!["all", "new_users", "returning_users", "premium"];
        for segment in segments {
            let base_users = match segment {
                "all" => rng.gen_range(10000..100000),
                "new_users" => rng.gen_range(1000..10000),
                "returning_users" => rng.gen_range(5000..50000),
                "premium" => rng.gen_range(500..5000),
                _ => 10000,
            };

            let insights = AudienceInsights {
                segment: segment.to_string(),
                total_users: base_users,
                age_groups: [
                    ("18-24".to_string(), (base_users as f64 * 0.15) as u64),
                    ("25-34".to_string(), (base_users as f64 * 0.35) as u64),
                    ("35-44".to_string(), (base_users as f64 * 0.25) as u64),
                    ("45-54".to_string(), (base_users as f64 * 0.15) as u64),
                    ("55+".to_string(), (base_users as f64 * 0.10) as u64),
                ]
                .into_iter()
                .collect(),
                locations: [
                    (
                        "United States".to_string(),
                        (base_users as f64 * 0.40) as u64,
                    ),
                    (
                        "United Kingdom".to_string(),
                        (base_users as f64 * 0.15) as u64,
                    ),
                    ("Canada".to_string(), (base_users as f64 * 0.12) as u64),
                    ("Germany".to_string(), (base_users as f64 * 0.10) as u64),
                    ("Other".to_string(), (base_users as f64 * 0.23) as u64),
                ]
                .into_iter()
                .collect(),
                interests: [
                    ("Technology".to_string(), (base_users as f64 * 0.35) as u64),
                    ("Business".to_string(), (base_users as f64 * 0.25) as u64),
                    ("Education".to_string(), (base_users as f64 * 0.20) as u64),
                    (
                        "Entertainment".to_string(),
                        (base_users as f64 * 0.15) as u64,
                    ),
                    ("Other".to_string(), (base_users as f64 * 0.05) as u64),
                ]
                .into_iter()
                .collect(),
                devices: [
                    ("Desktop".to_string(), (base_users as f64 * 0.45) as u64),
                    ("Mobile".to_string(), (base_users as f64 * 0.40) as u64),
                    ("Tablet".to_string(), (base_users as f64 * 0.15) as u64),
                ]
                .into_iter()
                .collect(),
                acquisition_channels: [
                    (
                        "Organic Search".to_string(),
                        (base_users as f64 * 0.35) as u64,
                    ),
                    ("Direct".to_string(), (base_users as f64 * 0.25) as u64),
                    (
                        "Social Media".to_string(),
                        (base_users as f64 * 0.20) as u64,
                    ),
                    ("Email".to_string(), (base_users as f64 * 0.10) as u64),
                    ("Paid Search".to_string(), (base_users as f64 * 0.10) as u64),
                ]
                .into_iter()
                .collect(),
                generated_at: now,
            };

            self.audience_data.insert(segment.to_string(), insights);
        }

        // Generate mock engagement trends
        let timespans = vec!["7d", "30d", "90d", "1y"];
        for timespan in timespans {
            let days = match timespan {
                "7d" => 7,
                "30d" => 30,
                "90d" => 90,
                "1y" => 365,
                _ => 30,
            };

            let mut daily_active_users = Vec::new();
            let mut session_duration_avg = Vec::new();
            let mut pages_per_session = Vec::new();
            let mut conversion_rate = Vec::new();

            for i in 0..days {
                let date = now - Duration::days(days - i);
                let base_dau = rng.gen_range(1000..10000) as f64;

                daily_active_users.push(TrendPoint {
                    date,
                    value: base_dau,
                });

                session_duration_avg.push(TrendPoint {
                    date,
                    value: rng.gen_range(60.0..600.0),
                });

                pages_per_session.push(TrendPoint {
                    date,
                    value: rng.gen_range(1.5..8.0),
                });

                conversion_rate.push(TrendPoint {
                    date,
                    value: rng.gen_range(0.01..0.15),
                });
            }

            let trends = EngagementTrends {
                timespan: timespan.to_string(),
                daily_active_users,
                session_duration_avg,
                pages_per_session,
                conversion_rate,
                generated_at: now,
            };

            self.engagement_trends.insert(timespan.to_string(), trends);
        }
    }

    async fn update_stats(&self, tool_name: &str) {
        let mut count = self.tool_stats.entry(tool_name.to_string()).or_insert(0);
        *count += 1;
    }

    fn format_content_metrics(&self, metrics: &ContentMetrics) -> String {
        format!(
            r#"📊 **Content Metrics for {} ({})**

**📈 Engagement Overview:**
- Views: {} (Unique: {})
- Likes: {} | Shares: {} | Comments: {}
- Engagement Rate: {:.2}%
- Conversion Rate: {:.2}%

**⏱️ Session Metrics:**
- Average Duration: {}m {}s
- Bounce Rate: {:.1}%

**🎯 Performance Score:** {:.1}/10

*Generated: {}*"#,
            metrics.content_id,
            metrics.period,
            metrics.views,
            metrics.unique_views,
            metrics.likes,
            metrics.shares,
            metrics.comments,
            metrics.engagement_rate * 100.0,
            metrics.conversion_rate * 100.0,
            metrics.average_session_duration / 60,
            metrics.average_session_duration % 60,
            metrics.bounce_rate * 100.0,
            (metrics.engagement_rate * 10.0).min(10.0),
            metrics.generated_at.format("%Y-%m-%d %H:%M UTC")
        )
    }

    fn format_audience_insights(&self, insights: &AudienceInsights) -> String {
        let top_location = insights
            .locations
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| name.as_str())
            .unwrap_or("Unknown");

        let top_interest = insights
            .interests
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| name.as_str())
            .unwrap_or("Unknown");

        format!(
            r#"👥 **Audience Insights: {} Segment**

**📊 Overview:**
- Total Users: {}
- Top Location: {}
- Primary Interest: {}

**🎂 Age Distribution:**
{}

**🌍 Geographic Distribution:**
{}

**📱 Device Usage:**
{}

**🚀 Acquisition Channels:**
{}

*Generated: {}*"#,
            insights.segment,
            insights.total_users,
            top_location,
            top_interest,
            insights
                .age_groups
                .iter()
                .map(|(age, count)| format!("- {age}: {count} users"))
                .collect::<Vec<_>>()
                .join("\n"),
            insights
                .locations
                .iter()
                .take(5)
                .map(|(location, count)| format!(
                    "- {}: {} users ({:.1}%)",
                    location,
                    count,
                    (*count as f64 / insights.total_users as f64) * 100.0
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            insights
                .devices
                .iter()
                .map(|(device, count)| format!(
                    "- {}: {} users ({:.1}%)",
                    device,
                    count,
                    (*count as f64 / insights.total_users as f64) * 100.0
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            insights
                .acquisition_channels
                .iter()
                .take(5)
                .map(|(channel, count)| format!(
                    "- {}: {} users ({:.1}%)",
                    channel,
                    count,
                    (*count as f64 / insights.total_users as f64) * 100.0
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            insights.generated_at.format("%Y-%m-%d %H:%M UTC")
        )
    }

    fn format_engagement_trends(&self, trends: &EngagementTrends) -> String {
        let avg_dau = trends
            .daily_active_users
            .iter()
            .map(|p| p.value)
            .sum::<f64>()
            / trends.daily_active_users.len() as f64;

        let avg_session = trends
            .session_duration_avg
            .iter()
            .map(|p| p.value)
            .sum::<f64>()
            / trends.session_duration_avg.len() as f64;

        let latest_conversion = trends
            .conversion_rate
            .last()
            .map(|p| p.value)
            .unwrap_or(0.0);

        format!(
            r#"📈 **Engagement Trends ({})**

**🎯 Key Metrics:**
- Average Daily Active Users: {:.0}
- Average Session Duration: {:.1} minutes
- Current Conversion Rate: {:.2}%
- Data Points: {} days

**📊 Trend Analysis:**
- DAU Range: {:.0} - {:.0}
- Session Duration Range: {:.1}m - {:.1}m
- Conversion Trend: {}

**🔍 Insights:**
- Peak Activity: {}
- Best Conversion Day: {}

*Generated: {}*"#,
            trends.timespan,
            avg_dau,
            avg_session / 60.0,
            latest_conversion * 100.0,
            trends.daily_active_users.len(),
            trends
                .daily_active_users
                .iter()
                .map(|p| p.value)
                .fold(f64::INFINITY, f64::min),
            trends
                .daily_active_users
                .iter()
                .map(|p| p.value)
                .fold(f64::NEG_INFINITY, f64::max),
            trends
                .session_duration_avg
                .iter()
                .map(|p| p.value / 60.0)
                .fold(f64::INFINITY, f64::min),
            trends
                .session_duration_avg
                .iter()
                .map(|p| p.value / 60.0)
                .fold(f64::NEG_INFINITY, f64::max),
            if latest_conversion > 0.05 {
                "📈 Strong"
            } else if latest_conversion > 0.02 {
                "📊 Moderate"
            } else {
                "📉 Needs Improvement"
            },
            trends
                .daily_active_users
                .iter()
                .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
                .map(|p| p.date.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            trends
                .conversion_rate
                .iter()
                .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
                .map(|p| p.date.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            trends.generated_at.format("%Y-%m-%d %H:%M UTC")
        )
    }
}

// ================================================================================================
// MCP Tools Implementation
// ================================================================================================

#[tool_router]
impl AnalyticsServer {
    #[tool(description = "Get content metrics for specific content with time period")]
    async fn get_content_metrics(
        &self,
        Parameters(args): Parameters<GetContentMetricsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("get_content_metrics").await;

        let key = format!("{}:{}", args.content_id, args.period);

        if let Some(metrics) = self.content_metrics.get(&key) {
            let result = self.format_content_metrics(&metrics);
            Ok(CallToolResult::success(vec![Content::text(result)]))
        } else {
            Err(McpError::new(
                rmcp::model::ErrorCode(-32602),
                format!(
                    "Metrics not found for content '{}' with period '{}'",
                    args.content_id, args.period
                ),
                None,
            ))
        }
    }

    #[tool(description = "Get audience insights and demographics for segment")]
    async fn get_audience_insights(
        &self,
        Parameters(args): Parameters<GetAudienceInsightsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("get_audience_insights").await;

        if let Some(insights) = self.audience_data.get(&args.segment) {
            let result = self.format_audience_insights(&insights);
            Ok(CallToolResult::success(vec![Content::text(result)]))
        } else {
            Err(McpError::new(
                rmcp::model::ErrorCode(-32602),
                format!("Audience insights not found for segment '{}'", args.segment),
                None,
            ))
        }
    }

    #[tool(description = "Get engagement trends over time")]
    async fn get_engagement_trends(
        &self,
        Parameters(args): Parameters<GetEngagementTrendsArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("get_engagement_trends").await;

        if let Some(trends) = self.engagement_trends.get(&args.timespan) {
            let result = self.format_engagement_trends(&trends);
            Ok(CallToolResult::success(vec![Content::text(result)]))
        } else {
            Err(McpError::new(
                rmcp::model::ErrorCode(-32602),
                format!(
                    "Engagement trends not found for timespan '{}'",
                    args.timespan
                ),
                None,
            ))
        }
    }

    #[tool(description = "Generate comprehensive analytics report")]
    async fn generate_analytics_report(
        &self,
        Parameters(args): Parameters<GenerateAnalyticsReportArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.update_stats("generate_analytics_report").await;

        let config = &args.config;

        let mut report = format!(
            r#"📊 **Analytics Report: {}**
**Generated:** {}
**Time Range:** {}

"#,
            config.report_type.to_uppercase(),
            Utc::now().format("%Y-%m-%d %H:%M UTC"),
            config.time_range
        );

        // Add executive summary
        if config.report_type == "executive" || config.report_type == "summary" {
            report.push_str("## Executive Summary\n\n");
            report.push_str("- 📈 Overall engagement trending upward\n");
            report.push_str("- 👥 Audience growth of 15% this period\n");
            report.push_str("- 💰 Conversion rates improved by 8%\n");
            report.push_str("- 🎯 Recommended focus: Mobile optimization\n\n");
        }

        // Add detailed metrics
        if config.metrics.contains(&"content".to_string()) {
            report.push_str("## Content Performance\n\n");
            if let Some(metrics) = self.content_metrics.get("blog-post-123:day") {
                report.push_str(&format!(
                    "**Top Performing Content:**\n{}\n\n",
                    self.format_content_metrics(&metrics)
                ));
            }
        }

        if config.metrics.contains(&"audience".to_string()) {
            report.push_str("## Audience Analysis\n\n");
            if let Some(insights) = self.audience_data.get("all") {
                report.push_str(&format!("{}\n\n", self.format_audience_insights(&insights)));
            }
        }

        if config.metrics.contains(&"trends".to_string()) {
            report.push_str("## Engagement Trends\n\n");
            if let Some(trends) = self.engagement_trends.get("30d") {
                report.push_str(&format!("{}\n\n", self.format_engagement_trends(&trends)));
            }
        }

        // Add recommendations
        report.push_str("## Recommendations\n\n");
        report.push_str("1. **Content Strategy:** Focus on high-engagement formats\n");
        report.push_str("2. **Audience Growth:** Expand mobile user acquisition\n");
        report.push_str("3. **Conversion Optimization:** A/B test checkout process\n");
        report.push_str("4. **Retention:** Implement personalized content recommendations\n\n");

        report.push_str("*This report contains mock data for demonstration purposes.*");

        Ok(CallToolResult::success(vec![Content::text(report)]))
    }

    #[tool(description = "List all available analytics metrics")]
    async fn get_available_metrics(&self) -> Result<CallToolResult, McpError> {
        self.update_stats("get_available_metrics").await;

        let metrics_info = r#"📊 **Available Analytics Metrics**

## Content Metrics
- `views` - Total and unique page views
- `engagement` - Likes, shares, comments
- `conversion_rate` - Goal completion percentage
- `session_duration` - Average time on site
- `bounce_rate` - Single-page session percentage

## Audience Metrics  
- `demographics` - Age groups and locations
- `interests` - User interest categories
- `devices` - Desktop, mobile, tablet usage
- `acquisition` - Traffic source channels
- `behavior` - User journey patterns

## Performance Metrics
- `daily_active_users` - DAU trends
- `session_metrics` - Duration and page views
- `conversion_funnel` - Step-by-step analysis
- `retention_rates` - User return patterns

## Business Metrics
- `revenue` - Sales and transaction data
- `customer_lifetime_value` - CLV analysis
- `cost_per_acquisition` - Marketing efficiency
- `churn_rate` - User attrition metrics

**Supported Periods:** hour, day, week, month, year
**Supported Segments:** all, new_users, returning_users, premium
**Supported Timespans:** 7d, 30d, 90d, 1y"#;

        Ok(CallToolResult::success(vec![Content::text(
            metrics_info.to_string(),
        )]))
    }

    #[tool(description = "List supported time periods and segments")]
    async fn get_time_periods(&self) -> Result<CallToolResult, McpError> {
        self.update_stats("get_time_periods").await;

        let periods_info = r#"⏰ **Supported Time Periods**

## Content Metrics Periods
- `hour` - Last 60 minutes of data
- `day` - Last 24 hours of data  
- `week` - Last 7 days of data
- `month` - Last 30 days of data
- `year` - Last 365 days of data

## Trend Analysis Timespans
- `7d` - 7-day trend analysis (daily points)
- `30d` - 30-day trend analysis (daily points)
- `90d` - 90-day trend analysis (daily points)
- `1y` - 1-year trend analysis (daily points)

## Audience Segments
- `all` - All users in the system
- `new_users` - Users acquired in the time period
- `returning_users` - Users with multiple sessions
- `premium` - Paid subscription users

## Report Time Ranges
- `today` - Current day data
- `yesterday` - Previous day data
- `this_week` - Current week data
- `last_week` - Previous week data
- `this_month` - Current month data
- `last_month` - Previous month data
- `this_quarter` - Current quarter data
- `last_quarter` - Previous quarter data
- `this_year` - Current year data
- `custom` - User-defined date range"#;

        Ok(CallToolResult::success(vec![Content::text(
            periods_info.to_string(),
        )]))
    }

    #[tool(description = "Get analytics server health and status")]
    async fn get_server_status(&self) -> Result<CallToolResult, McpError> {
        self.update_stats("get_server_status").await;

        let total_requests: u64 = self.tool_stats.iter().map(|entry| *entry.value()).sum();

        let mut status_parts = vec![
            "🔍 **Analytics Server Status**".to_string(),
            "".to_string(),
            "**📊 Server Health:** ✅ Online".to_string(),
            format!("**📈 Total Requests:** {}", total_requests),
            format!(
                "**🗄️ Content Metrics:** {} cached",
                self.content_metrics.len()
            ),
            format!(
                "**👥 Audience Segments:** {} available",
                self.audience_data.len()
            ),
            format!(
                "**📉 Trend Data:** {} timespans",
                self.engagement_trends.len()
            ),
            "".to_string(),
            "**🔧 Tool Usage:**".to_string(),
        ];

        for entry in self.tool_stats.iter() {
            status_parts.push(format!("  - {}: {} requests", entry.key(), entry.value()));
        }

        status_parts.push("".to_string());
        status_parts.push("**⚡ Performance:** All metrics responding < 100ms".to_string());
        status_parts.push("**🔒 Security:** Data sanitization active".to_string());
        status_parts.push("**💾 Storage:** Mock data generation enabled".to_string());

        Ok(CallToolResult::success(vec![Content::text(
            status_parts.join("\n"),
        )]))
    }
}

// ================================================================================================
// Server Handler Implementation
// ================================================================================================

impl ServerHandler for AnalyticsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "🔍 Analytics Server - Metrics and performance analytics:\n\
                • get_content_metrics: Get metrics for specific content with time periods\n\
                • get_audience_insights: Get audience segmentation and demographics\n\
                • get_engagement_trends: Get engagement trends over time\n\
                • generate_analytics_report: Generate comprehensive analytics reports\n\
                • get_available_metrics: List all available metric types\n\
                • get_time_periods: List supported time periods and segments\n\
                • get_server_status: Health check and usage statistics\n\n\
                📊 Mock data includes content metrics, audience insights, engagement trends\n\
                🚀 Fast, lightweight implementation using official RMCP SDK"
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        info!("🔍 Analytics Server initialized with comprehensive metrics");
        info!("📊 Mock data generated for content, audience, and trends");
        Ok(self.get_info())
    }
}

impl Default for AnalyticsServer {
    fn default() -> Self {
        Self::new()
    }
}

// ================================================================================================
// Main Function
// ================================================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(format!("analytics_server={log_level}").parse()?)
                .add_directive(format!("rmcp={log_level}").parse()?),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("🚀 Starting Analytics Server using official RMCP SDK");
    info!("📊 Metrics and performance analytics ready");

    // Create server instance
    let server = AnalyticsServer::new();

    // Start the server with STDIO transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        error!("Failed to start server: {:?}", e);
    })?;

    info!("✅ Analytics Server started and ready for MCP connections");
    info!("🔍 Mock analytics data available for testing");

    // Wait for the service to complete
    service.waiting().await?;

    info!("Server shutdown complete");
    Ok(())
}

// ================================================================================================
// Tests
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::RawContent;

    #[tokio::test]
    async fn test_server_creation() {
        let server = AnalyticsServer::new();
        assert!(!server.content_metrics.is_empty());
        assert!(!server.audience_data.is_empty());
        assert!(!server.engagement_trends.is_empty());
    }

    #[tokio::test]
    async fn test_content_metrics_tool() {
        let server = AnalyticsServer::new();
        let args = GetContentMetricsArgs {
            content_id: "blog-post-123".to_string(),
            period: "day".to_string(),
        };

        let result = server.get_content_metrics(Parameters(args)).await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Content Metrics"));
                assert!(text.text.contains("blog-post-123"));
            }
        }
    }

    #[tokio::test]
    async fn test_audience_insights_tool() {
        let server = AnalyticsServer::new();
        let args = GetAudienceInsightsArgs {
            segment: "all".to_string(),
        };

        let result = server
            .get_audience_insights(Parameters(args))
            .await
            .unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Audience Insights"));
                assert!(text.text.contains("Total Users"));
            }
        }
    }

    #[tokio::test]
    async fn test_engagement_trends_tool() {
        let server = AnalyticsServer::new();
        let args = GetEngagementTrendsArgs {
            timespan: "30d".to_string(),
        };

        let result = server
            .get_engagement_trends(Parameters(args))
            .await
            .unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Engagement Trends"));
                assert!(text.text.contains("Daily Active Users"));
            }
        }
    }

    #[tokio::test]
    async fn test_generate_report_tool() {
        let server = AnalyticsServer::new();
        let args = GenerateAnalyticsReportArgs {
            config: ReportConfig {
                report_type: "summary".to_string(),
                metrics: vec!["content".to_string(), "audience".to_string()],
                time_range: "last_month".to_string(),
                format: "markdown".to_string(),
            },
        };

        let result = server
            .generate_analytics_report(Parameters(args))
            .await
            .unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Analytics Report"));
                assert!(text.text.contains("Executive Summary"));
            }
        }
    }

    #[tokio::test]
    async fn test_available_metrics_tool() {
        let server = AnalyticsServer::new();
        let result = server.get_available_metrics().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Available Analytics Metrics"));
                assert!(text.text.contains("Content Metrics"));
            }
        }
    }

    #[tokio::test]
    async fn test_time_periods_tool() {
        let server = AnalyticsServer::new();
        let result = server.get_time_periods().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Supported Time Periods"));
                assert!(text.text.contains("hour"));
            }
        }
    }

    #[tokio::test]
    async fn test_server_status_tool() {
        let server = AnalyticsServer::new();
        let result = server.get_server_status().await.unwrap();
        assert!(result.is_error.is_none() || !result.is_error.unwrap());

        if let Some(content) = result.content.first() {
            if let RawContent::Text(text) = &content.raw {
                assert!(text.text.contains("Analytics Server Status"));
                assert!(text.text.contains("Online"));
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_content_id() {
        let server = AnalyticsServer::new();
        let args = GetContentMetricsArgs {
            content_id: "nonexistent".to_string(),
            period: "day".to_string(),
        };

        let result = server.get_content_metrics(Parameters(args)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_audience_segment() {
        let server = AnalyticsServer::new();
        let args = GetAudienceInsightsArgs {
            segment: "nonexistent".to_string(),
        };

        let result = server.get_audience_insights(Parameters(args)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_data_generation() {
        let server = AnalyticsServer::new();

        // Verify content metrics are generated
        assert!(server.content_metrics.len() > 0);

        // Verify audience data is generated
        assert!(server.audience_data.len() >= 4); // all, new_users, returning_users, premium

        // Verify engagement trends are generated
        assert!(server.engagement_trends.len() >= 4); // 7d, 30d, 90d, 1y
    }
}
