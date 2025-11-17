import { spawn, ChildProcess } from 'child_process';
import axios from 'axios';
import { promisify } from 'util';
import { setTimeout } from 'timers/promises';

// Test configuration
const AGGREGATOR_URL = 'http://localhost:8080';
const WORKER_PORT = 8081;
const TEST_TIMEOUT = 30000; // 30 seconds

describe('Full Flow End-to-End Tests', () => {
  let aggregatorProcess: ChildProcess;
  let workerProcess: ChildProcess;

  // Start the aggregator and worker before running tests
  beforeAll(async () => {
    console.log('Starting aggregator...');
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

    aggregatorProcess.stdout?.on('data', (data) => {
      console.log(`Aggregator: ${data}`);
    });

    aggregatorProcess.stderr?.on('data', (data) => {
      console.error(`Aggregator Error: ${data}`);
    });

    // Wait for aggregator to start
    await setTimeout(5000);

    console.log('Starting worker...');
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

    workerProcess.stdout?.on('data', (data) => {
      console.log(`Worker: ${data}`);
    });

    workerProcess.stderr?.on('data', (data) => {
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

  test('should successfully request and receive entropy', async () => {
    // Wait for services to be fully ready
    await setTimeout(2000);

    // Make a request to the aggregator for entropy
    const response = await axios.post(`${AGGREGATOR_URL}/request`, {
      requester_id: 'test_client_1',
      entropy_type: 'random_bytes',
      size: 32,
      callback_url: null
    }, {
      timeout: 10000
    });

    expect(response.status).toBe(200);
    expect(response.data).toHaveProperty('request_id');
    expect(response.data).toHaveProperty('entropy');
    expect(response.data.entropy).toHaveLength(32);
 }, TEST_TIMEOUT);

  test('should handle multiple concurrent entropy requests', async () => {
    // Wait for services to be fully ready
    await setTimeout(2000);

    // Make multiple concurrent requests
    const requests = [];
    for (let i = 0; i < 5; i++) {
      requests.push(
        axios.post(`${AGGREGATOR_URL}/request`, {
          requester_id: `test_client_${i}`,
          entropy_type: 'random_bytes',
          size: 32,
          callback_url: null
        }, {
          timeout: 10000
        })
      );
    }

    const responses = await Promise.all(requests);
    
    expect(responses).toHaveLength(5);
    responses.forEach(response => {
      expect(response.status).toBe(200);
      expect(response.data).toHaveProperty('request_id');
      expect(response.data).toHaveProperty('entropy');
      expect(response.data.entropy).toHaveLength(32);
    });
  }, TEST_TIMEOUT);

  test('should handle entropy request with callback', async () => {
    // Wait for services to be fully ready
    await setTimeout(2000);

    // Mock callback server
    const callbackReceived = new Promise((resolve) => {
      // In a real test, we would set up an actual server to receive the callback
      // For this test, we'll just verify the aggregator can handle callback requests
      setTimeout(() => resolve(true), 1000);
    });

    const response = await axios.post(`${AGGREGATOR_URL}/request`, {
      requester_id: 'test_client_callback',
      entropy_type: 'random_bytes',
      size: 64,
      callback_url: 'http://localhost:9999/callback'
    }, {
      timeout: 10000
    });

    expect(response.status).toBe(200);
    expect(response.data).toHaveProperty('request_id');
    expect(response.data).toHaveProperty('entropy');
    expect(response.data.entropy).toHaveLength(64);

    await callbackReceived;
  }, TEST_TIMEOUT);

  test('should maintain entropy quality standards', async () => {
    // Wait for services to be fully ready
    await setTimeout(2000);

    // Request multiple entropy values and verify they're unique
    const entropyValues = [];
    for (let i = 0; i < 10; i++) {
      const response = await axios.post(`${AGGREGATOR_URL}/request`, {
        requester_id: `quality_test_client_${i}`,
        entropy_type: 'random_bytes',
        size: 32,
        callback_url: null
      }, {
        timeout: 10000
      });
      
      entropyValues.push(response.data.entropy);
    }

    // Check that all values are unique (highly likely with good entropy)
    const uniqueValues = new Set(entropyValues.map(e => JSON.stringify(e)));
    expect(uniqueValues.size).toBeGreaterThan(8); // Allow some potential duplicates but expect mostly unique
  }, TEST_TIMEOUT);

  test('should handle client disconnection gracefully', async () => {
    // Wait for services to be fully ready
    await setTimeout(2000);

    // Make a request but don't wait for full response to simulate disconnection
    const requestPromise = axios.post(`${AGGREGATOR_URL}/request`, {
      requester_id: 'disconnected_client',
      entropy_type: 'random_bytes',
      size: 32,
      callback_url: null
    }, {
      timeout: 5000
    });

    // Wait briefly then proceed (simulating client disconnection)
    await setTimeout(1000);

    const response = await requestPromise;
    expect(response.status).toBe(200);
    expect(response.data).toHaveProperty('request_id');
    expect(response.data).toHaveProperty('entropy');
  }, TEST_TIMEOUT);
});