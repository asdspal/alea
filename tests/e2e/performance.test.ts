import { spawn, ChildProcess } from 'child_process';
import axios from 'axios';
import { setTimeout } from 'timers/promises';

// Test configuration
const AGGREGATOR_URL = 'http://localhost:8080';
const WORKER_PORT = 8081;
const TEST_TIMEOUT = 60000; // 60 seconds

describe('Performance Tests', () => {
  let aggregatorProcess: ChildProcess;
  let workerProcess: ChildProcess;

  // Start the aggregator and worker before running tests
  beforeAll(async () => {
    console.log('Starting aggregator for performance tests...');
    aggregatorProcess = spawn('cargo', ['run', '--bin', 'entropy-aggregator'], {
      cwd: './alea/entropy-aggregator',
      env: {
        ...process.env,
        ALEA_TEST_MODE: 'true',
        ALEA_LOG_LEVEL: 'info',
        ALEA_NETWORK_MODE: 'testing',
        SGX_MODE: 'SW',
        ALEA_USE_MOCK_SGX: 'true',
        ALEA_TEST_AGGREGATOR_PORT: '8080',
        ALEA_TEST_WORKER_PORT: '8081'
      }
    });

    aggregatorProcess.stdout?.on('data', (data: any) => {
      console.log(`Aggregator: ${data}`);
    });

    aggregatorProcess.stderr?.on('data', (data: any) => {
      console.error(`Aggregator Error: ${data}`);
    });

    // Wait for aggregator to start
    await setTimeout(5000);

    console.log('Starting worker for performance tests...');
    workerProcess = spawn('cargo', ['run', '--bin', 'entropy-worker'], {
      cwd: './alea/entropy-worker',
      env: {
        ...process.env,
        ALEA_TEST_MODE: 'true',
        ALEA_LOG_LEVEL: 'info',
        ALEA_NETWORK_MODE: 'testing',
        SGX_MODE: 'SW',
        ALEA_USE_MOCK_SGX: 'true',
        ALEA_TEST_AGGREGATOR_PORT: '8080',
        ALEA_TEST_WORKER_PORT: '8081'
      }
    });

    workerProcess.stdout?.on('data', (data: any) => {
      console.log(`Worker: ${data}`);
    });

    workerProcess.stderr?.on('data', (data: any) => {
      console.error(`Worker Error: ${data}`);
    });

    // Wait for worker to start and connect to aggregator
    await setTimeout(5000);
  }, 60000); // 60 seconds for startup

 // Clean up after tests
  afterAll(async () => {
    if (aggregatorProcess) {
      aggregatorProcess.kill();
    }
    if (workerProcess) {
      workerProcess.kill();
    }
  }, 10000);

  test('should handle minimum throughput requirement (REQ-PERF-001)', async () => {
    // Wait for services to be fully ready
    await setTimeout(2000);

    const startTime = Date.now();
    const requestCount = 100;
    const requests = [];

    // Send multiple requests in parallel
    for (let i = 0; i < requestCount; i++) {
      requests.push(
        axios.post(`${AGGREGATOR_URL}/request`, {
          requester_id: `perf_client_${i}`,
          entropy_type: 'random_bytes',
          size: 32,
          callback_url: null
        }, {
          timeout: 10000
        })
      );
    }

    const responses = await Promise.all(requests);
    const endTime = Date.now();
    const duration = endTime - startTime;

    // REQ-PERF-001: System should handle at least 50 requests per second
    const requestsPerSecond = requestCount / (duration / 1000);
    
    console.log(`Processed ${requestCount} requests in ${duration}ms (${requestsPerSecond.toFixed(2)} req/s)`);
    
    expect(responses).toHaveLength(requestCount);
    expect(requestsPerSecond).toBeGreaterThan(50); // Minimum requirement

    // Verify all responses are valid
    responses.forEach(response => {
      expect(response.status).toBe(200);
      expect(response.data).toHaveProperty('request_id');
      expect(response.data).toHaveProperty('entropy');
      expect(response.data.entropy).toHaveLength(32);
    });
  }, TEST_TIMEOUT);

  test('should meet latency requirements (REQ-PERF-002)', async () => {
    // Wait for services to be fully ready
    await setTimeout(2000);

    const latencyMeasurements = [];
    const requestCount = 50;

    // Measure individual request latencies
    for (let i = 0; i < requestCount; i++) {
      const startTime = Date.now();
      
      const response = await axios.post(`${AGGREGATOR_URL}/request`, {
        requester_id: `latency_client_${i}`,
        entropy_type: 'random_bytes',
        size: 32,
        callback_url: null
      }, {
        timeout: 10000
      });

      const endTime = Date.now();
      const latency = endTime - startTime;
      latencyMeasurements.push(latency);

      expect(response.status).toBe(200);
      expect(response.data).toHaveProperty('request_id');
      expect(response.data).toHaveProperty('entropy');
    }

    // Calculate average and p95 latency
    const avgLatency = latencyMeasurements.reduce((a, b) => a + b, 0) / latencyMeasurements.length;
    const sortedLatencies = [...latencyMeasurements].sort((a, b) => a - b);
    const p95Latency = sortedLatencies[Math.floor(sortedLatencies.length * 0.95)];
    
    console.log(`Average latency: ${avgLatency.toFixed(2)}ms`);
    console.log(`P95 latency: ${p95Latency}ms`);

    // REQ-PERF-002: 95th percentile latency should be under 200ms
    expect(p95Latency).toBeLessThan(200);
  }, TEST_TIMEOUT);

  test('should maintain performance under load (REQ-PERF-003)', async () => {
    // Wait for services to be fully ready
    await setTimeout(2000);

    const requestCount = 200;
    const requests = [];
    const startTime = Date.now();

    // Send requests in batches to simulate sustained load
    for (let batch = 0; batch < 10; batch++) {
      const batchRequests = [];
      for (let i = 0; i < 20; i++) {
        const idx = batch * 20 + i;
        batchRequests.push(
          axios.post(`${AGGREGATOR_URL}/request`, {
            requester_id: `load_client_${idx}`,
            entropy_type: 'random_bytes',
            size: 32,
            callback_url: null
          }, {
            timeout: 10000
          })
        );
      }
      
      // Wait briefly between batches to avoid overwhelming the system
      await Promise.all(batchRequests);
      requests.push(...batchRequests);
      await setTimeout(100); // Small delay between batches
    }

    const responses = await Promise.all(requests.map(p => p));
    const endTime = Date.now();
    const duration = endTime - startTime;

    // Calculate throughput
    const requestsPerSecond = requestCount / (duration / 1000);
    
    console.log(`Processed ${requestCount} requests in ${duration}ms (${requestsPerSecond.toFixed(2)} req/s) under load`);

    // REQ-PERF-003: System should maintain at least 40 requests per second under load
    expect(responses).toHaveLength(requestCount);
    expect(requestsPerSecond).toBeGreaterThan(40);

    // Verify all responses are valid
    responses.forEach(response => {
      expect(response.status).toBe(200);
      expect(response.data).toHaveProperty('request_id');
      expect(response.data).toHaveProperty('entropy');
      expect(response.data.entropy).toHaveLength(32);
    });
  }, TEST_TIMEOUT);

  test('should maintain entropy quality under performance load', async () => {
    // Wait for services to be fully ready
    await setTimeout(2000);

    // Request multiple entropy values under load and verify they're unique
    const entropyValues = [];
    const requestCount = 100;

    // Send requests in parallel to test quality under load
    const requests = [];
    for (let i = 0; i < requestCount; i++) {
      requests.push(
        axios.post(`${AGGREGATOR_URL}/request`, {
          requester_id: `quality_load_client_${i}`,
          entropy_type: 'random_bytes',
          size: 32,
          callback_url: null
        }, {
          timeout: 10000
        })
      );
    }

    const responses = await Promise.all(requests);
    
    // Extract entropy values
    for (const response of responses) {
      expect(response.status).toBe(200);
      entropyValues.push(response.data.entropy);
    }

    // Check that entropy values are unique (highly likely with good entropy)
    const uniqueValues = new Set(entropyValues.map(e => JSON.stringify(e)));
    
    // Under load, we expect high entropy quality - at least 95% unique values
    const uniquenessRatio = uniqueValues.size / entropyValues.length;
    console.log(`Entropy uniqueness ratio under load: ${uniquenessRatio.toFixed(3)}`);
    
    expect(uniquenessRatio).toBeGreaterThan(0.95);
  }, TEST_TIMEOUT);
});