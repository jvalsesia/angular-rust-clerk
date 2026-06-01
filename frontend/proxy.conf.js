const fs = require('fs');
const path = require('path');

let backendUrl = 'http://localhost:3000';

try {
  const envPath = path.resolve(__dirname, '.env');
  if (fs.existsSync(envPath)) {
    const envContent = fs.readFileSync(envPath, 'utf8');
    const match = envContent.match(/^BACKEND_URL=(.+)$/m);
    if (match) {
      backendUrl = match[1].trim();
    }
  }
} catch (e) {
  console.warn('Failed to parse .env file in proxy config:', e);
}

// Environment variable overrides .env file
if (process.env.BACKEND_URL) {
  backendUrl = process.env.BACKEND_URL;
}

console.log(`[Proxy] Routing /api to: ${backendUrl}`);

const PROXY_CONFIG = {
  "/api": {
    "target": backendUrl,
    "secure": false,
    "changeOrigin": true
  }
};

module.exports = PROXY_CONFIG;
