module.exports = {
  apps: [
    {
      name: 'dautruongvui_api_8030',
      script: '/opt/apps/dautruongvui-be/mcp-dautruongvui-be',
      args: '--mode http-stream --bind 0.0.0.0:8030',
      cwd: '/opt/apps/dautruongvui-be',
      exec_mode: 'fork',
      instances: 1,
      autorestart: true,
      watch: false,
      max_memory_restart: '256M',
      env: {
        PORT: 8030,
        HOST: '0.0.0.0',
        NODE_ENV: 'production',
        RUST_LOG: 'info',
        JWT_SECRET: 'aivaAPI',
        POSTGREST_URL: 'http://localhost:3001',
        DB_TABLE_PREFIX: 'dtv_',
      },
      error_file: '/opt/apps/dautruongvui-be/logs/error.log',
      out_file: '/opt/apps/dautruongvui-be/logs/out.log',
      merge_logs: true,
      log_date_format: 'YYYY-MM-DD HH:mm:ss Z',
    },
  ],
};