#!/usr/bin/env node

/**
 * Usage query script.
 * Determines whether to call the Z.ai or ZHIPU endpoint based on ANTHROPIC_BASE_URL
 * and authenticates with ANTHROPIC_AUTH_TOKEN.
 */

import https from 'https';

// Read environment variables
const baseUrl = 'https://open.bigmodel.cn/api/anthropic';
const authToken ='';

if (!authToken) {
  console.error('Error: ANTHROPIC_AUTH_TOKEN is not set');
  console.error('');
  console.error('Set the environment variable and retry:');
  console.error('  export ANTHROPIC_AUTH_TOKEN="your-token-here"');
  process.exit(1);
}

// Validate ANTHROPIC_BASE_URL
if (!baseUrl) {
  console.error('Error: ANTHROPIC_BASE_URL is not set');
  console.error('');
  console.error('Set the environment variable and retry:');
  console.error('  export ANTHROPIC_BASE_URL="https://api.z.ai/api/anthropic"');
  console.error('  or');
  console.error('  export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"');
  process.exit(1);
}

// Determine which platform to use
let platform;
let modelUsageUrl;
let toolUsageUrl;
let quotaLimitUrl;

// Extract the base domain from ANTHROPIC_BASE_URL
const parsedBaseUrl = new URL(baseUrl);
const baseDomain = `${parsedBaseUrl.protocol}//${parsedBaseUrl.host}`;

if (baseUrl.includes('api.z.ai')) {
  platform = 'ZAI';
  modelUsageUrl = `${baseDomain}/api/monitor/usage/model-usage`;
  toolUsageUrl = `${baseDomain}/api/monitor/usage/tool-usage`;
  quotaLimitUrl = `${baseDomain}/api/monitor/usage/quota/limit`;
} else if (baseUrl.includes('open.bigmodel.cn') || baseUrl.includes('dev.bigmodel.cn')) {
  platform = 'ZHIPU';
  modelUsageUrl = `${baseDomain}/api/monitor/usage/model-usage`;
  toolUsageUrl = `${baseDomain}/api/monitor/usage/tool-usage`;
  quotaLimitUrl = `${baseDomain}/api/monitor/usage/quota/limit`;
} else {
  console.error('Error: Unrecognized ANTHROPIC_BASE_URL:', baseUrl);
  console.error('');
  console.error('Supported values:');
  console.error('  - https://api.z.ai/api/anthropic');
  console.error('  - https://open.bigmodel.cn/api/anthropic');
  process.exit(1);
}

console.log(`Platform: ${platform}`);
console.log('');
// Time window: from yesterday at the current hour (HH:00:00) to today at the current hour end (HH:59:59).
const now = new Date();
const startDate = new Date(now.getFullYear(), now.getMonth(), now.getDate() - 1, now.getHours(), 0, 0, 0);
const endDate = new Date(now.getFullYear(), now.getMonth(), now.getDate(), now.getHours(), 59, 59, 999);

// Format dates as yyyy-MM-dd HH:mm:ss
const formatDateTime = (date) => {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
  const seconds = String(date.getSeconds()).padStart(2, '0');
  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
};

const startTime = formatDateTime(startDate);
const endtime = formatDateTime(endDate);

// Properly encode query parameters
const queryParams = `?startTime=${encodeURIComponent(startTime)}&endTime=${encodeURIComponent(endtime)}`;

/**
 * Identify subscription plan based on token usage limit
 * @param {number} usage - Token usage limit value
 * @returns {string} Plan name (Lite/Pro/Max/Unknown)
 */
const identifyPlan = (usage) => {
  const PRO_LIMIT = 200000000;  // 2亿 (Pro)
  const LITE_LIMIT = PRO_LIMIT / 5;  // 4千万 (Pro的1/5)
  const MAX_LIMIT = PRO_LIMIT * 4;   // 8亿 (Pro的4倍)

  if (usage === LITE_LIMIT) {
    return 'Lite';
  } else if (usage === PRO_LIMIT) {
    return 'Pro';
  } else if (usage === MAX_LIMIT) {
    return 'Max';
  } else {
    return 'Unknown';
  }
};

/**
 * Format large numbers with thousand separators
 * @param {number} num - Number to format
 * @returns {string} Formatted number string
 */
const formatNumber = (num) => {
  return num.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
};

const processQuotaLimit = (data) => {
  if (!data || !data.limits) return data;

  data.limits = data.limits.map(item => {
    const processed = { ...item };

    if (item.type === 'TOKENS_LIMIT') {
      // Identify subscription plan
      const plan = identifyPlan(item.usage);

      processed.type = `Token 用量 (${item.number} ${item.unit === 3 ? '小时' : '天'})`;
      processed.plan = plan;
      processed.planName = `套餐: ${plan}`;
      processed.usageFormatted = formatNumber(item.usage);
      processed.currentValueFormatted = formatNumber(item.currentValue);
      processed.remainingFormatted = formatNumber(item.remaining);
      processed.percentage = item.percentage;

      // Check if API provides reset time
      if (item.nextResetTime) {
        processed.nextResetTime = item.nextResetTime;
        processed.nextResetTimeReadable = new Date(item.nextResetTime).toLocaleString('zh-CN', { timeZone: 'Asia/Shanghai' });
      }
    }
    if (item.type === 'TIME_LIMIT') {
      processed.type = `MCP 用量 (${item.number} ${item.unit === 5 ? '个月' : '天'})`;
      processed.percentage = item.percentage;
      processed.currentUsage = item.currentValue;
      processed.total = item.usage;
      processed.remaining = item.remaining;
      processed.usageDetails = item.usageDetails;

      // Check if API provides reset time
      if (item.nextResetTime) {
        processed.nextResetTime = item.nextResetTime;
        processed.nextResetTimeReadable = new Date(item.nextResetTime).toLocaleString('zh-CN', { timeZone: 'Asia/Shanghai' });
      }
    }

    return processed;
  });

  // Check for global reset time
  if (data.nextResetTime) {
    data.nextResetTime = data.nextResetTime;
    data.nextResetTimeReadable = new Date(data.nextResetTime).toLocaleString('zh-CN', { timeZone: 'Asia/Shanghai' });
  }

  return data;
};

const queryUsage = (apiUrl, label, appendQueryParams = true, postProcessor = null) => {
  return new Promise((resolve, reject) => {
    const parsedUrl = new URL(apiUrl);
    const options = {
      hostname: parsedUrl.hostname,
      port: 443,
      path: parsedUrl.pathname + (appendQueryParams ? queryParams : ''),
      method: 'GET',
      headers: {
        'Authorization': authToken,
        'Accept-Language': 'en-US,en',
        'Content-Type': 'application/json'
      }
    };

    const req = https.request(options, (res) => {
      let data = '';

      res.on('data', (chunk) => {
        data += chunk;
      });

      res.on('end', () => {
        if (res.statusCode !== 200) {
          return reject(new Error(`[${label}] HTTP ${res.statusCode}\n${data}`));
        }

        console.log(`${label} data:`);
        console.log('');

        try {
          const json = JSON.parse(data);
          // Log full response for debugging
          console.log('Full API Response:');
          console.log(JSON.stringify(json, null, 2));
          console.log('');

          let outputData = json.data || json;
          if (postProcessor && json.data) {
            outputData = postProcessor(json.data);
          }
          console.log('Processed Data:');
          console.log(JSON.stringify(outputData, null, 2));
        } catch (e) {
          console.log('Response body:');
          console.log(data);
        }

        console.log('');
        resolve();
      });
    });

    req.on('error', (error) => {
      reject(error);
    });

    req.end();
  });
};

const run = async () => {
  await queryUsage(modelUsageUrl, 'Model usage');
  await queryUsage(toolUsageUrl, 'Tool usage');
  await queryUsage(quotaLimitUrl, 'Quota limit', false, processQuotaLimit);
};

run().catch((error) => {
  console.error('Request failed:', error.message);
  process.exit(1);
});
