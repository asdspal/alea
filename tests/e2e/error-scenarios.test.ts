import { spawn, ChildProcess } from 'child_process';
import axios from 'axios';
// Use global setTimeout for compatibility with Node.js environment
// This replaces the problematic timers/promises import that causes TS2307 error
const delay = (ms: number): Promise<void> => new Promise(resolve => setTimeout(resolve, ms));

// Test configuration
const AGGREGATOR_URL = 'http://localhost:8080';
const WORKER_PORT = 8081;
const TEST_TIMEOUT = 30000; // 30 seconds

describe('Error Scenario Tests', () => {
  let aggregatorProcess: ChildProcess;
  let workerProcess: ChildProcess;

  // Start the aggregator and worker before running tests
  beforeAll(async () => {
    console.log('Starting aggregator for error scenario tests...');
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
    await delay(5000);

    console.log('Starting worker for error scenario tests...');
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
    await delay(5000);
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

  test('should handle invalid request parameters gracefully', async () => {
    // Wait for services to be fully ready
    await delay(2000);

    // Test with invalid entropy type
    try {
      const response = await axios.post(`${AGGREGATOR_URL}/request`, {
        requester_id: 'invalid_client',
        entropy_type: 'invalid_type',
        size: 32,
        callback_url: null
      }, {
        timeout: 5000
      });
      
      // Should return an error status
      expect(response.status).toBeGreaterThanOrEqual(400);
    } catch (error: any) {
      // If request fails (which is expected), check the error
      expect(error.response.status).toBeGreaterThanOrEqual(400);
    }

    // Test with invalid size parameter
    try {
      const response = await axios.post(`${AGGREGATOR_URL}/request`, {
        requester_id: 'invalid_size_client',
        entropy_type: 'random_bytes',
        size: -1, // Invalid size
        callback_url: null
      }, {
        timeout: 5000
      });
      
      expect(response.status).toBeGreaterThanOrEqual(400);
    } catch (error: any) {
      expect(error.response.status).toBeGreaterThanOrEqual(400);
    }

    // Test with extremely large size
    try {
      const response = await axios.post(`${AGGREGATOR_URL}/request`, {
        requester_id: 'large_size_client',
        entropy_type: 'random_bytes',
        size: 1000000, // Very large size
        callback_url: null
      }, {
        timeout: 10000
      });
      
      expect(response.status).toBeGreaterThanOrEqual(400); // Or server might process it but with limits
    } catch (error: any) {
      expect(error.response.status).toBeGreaterThanOrEqual(400);
    }
  }, TEST_TIMEOUT);

  test('should handle aggregator restart gracefully', async () => {
    // Wait for services to be fully ready
    await delay(2000);

    // Make a successful request first
    const initialResponse = await axios.post(`${AGGREGATOR_URL}/request`, {
      requester_id: 'restart_client_1',
      entropy_type: 'random_bytes',
      size: 32,
      callback_url: null
    }, {
      timeout: 10000
    });

    expect(initialResponse.status).toBe(200);
    expect(initialResponse.data).toHaveProperty('request_id');
    expect(initialResponse.data).toHaveProperty('entropy');

    // Restart the aggregator process
    if (aggregatorProcess) {
      aggregatorProcess.kill();
    }

    // Wait for process to fully terminate
    await delay(3000);

    // Start the aggregator again
    console.log('Restarting aggregator...');
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

    // Wait for aggregator to restart
    await delay(8000);

    // Make another request after restart
    const afterRestartResponse = await axios.post(`${AGGREGATOR_URL}/request`, {
      requester_id: 'restart_client_2',
      entropy_type: 'random_bytes',
      size: 32,
      callback_url: null
    }, {
      timeout: 10000
    });

    expect(afterRestartResponse.status).toBe(200);
    expect(afterRestartResponse.data).toHaveProperty('request_id');
    expect(afterRestartResponse.data).toHaveProperty('entropy');
  }, TEST_TIMEOUT);

  test('should handle worker failure and recovery', async () => {
    // Wait for services to be fully ready
    await delay(2000);

    // Make a successful request first
    const initialResponse = await axios.post(`${AGGREGATOR_URL}/request`, {
      requester_id: 'worker_failure_client_1',
      entropy_type: 'random_bytes',
      size: 32,
      callback_url: null
    }, {
      timeout: 10000
    });

    expect(initialResponse.status).toBe(200);
    expect(initialResponse.data).toHaveProperty('request_id');
    expect(initialResponse.data).toHaveProperty('entropy');

    // Kill the worker process
    if (workerProcess) {
      console.log('Killing worker process...');
      workerProcess.kill();
    }

    // Wait briefly
    await delay(2000);

    // Try to make a request while worker is down
    try {
      await axios.post(`${AGGREGATOR_URL}/request`, {
        requester_id: 'worker_failure_client_2',
        entropy_type: 'random_bytes',
        size: 32,
        callback_url: null
      }, {
        timeout: 10000
      });
      
      // If the request succeeds, it means the system handled the worker failure gracefully
      // and possibly queued the request or used cached entropy
    } catch (error: any) {
      // If the request fails, that's also valid behavior as long as it fails gracefully
      console.log(`Request failed as expected when worker is down: ${error.message}`);
    }

    // Restart the worker
    console.log('Restarting worker process...');
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

    // Wait for worker to restart
    await delay(8000);

    // Make a request after worker restart
    const afterRestartResponse = await axios.post(`${AGGREGATOR_URL}/request`, {
      requester_id: 'worker_failure_client_3',
      entropy_type: 'random_bytes',
      size: 32,
      callback_url: null
    }, {
      timeout: 10000
    });

    expect(afterRestartResponse.status).toBe(200);
    expect(afterRestartResponse.data).toHaveProperty('request_id');
    expect(afterRestartResponse.data).toHaveProperty('entropy');
  }, TEST_TIMEOUT);

  test('should handle network timeout scenarios', async () => {
    // Wait for services to be fully ready
    await delay(2000);

    // Make a request with a very short timeout to simulate network issues
    try {
      const response = await axios.post(`${AGGREGATOR_URL}/request`, {
        requester_id: 'timeout_client',
        entropy_type: 'random_bytes',
        size: 32,
        callback_url: null
      }, {
        timeout: 100 // Very short timeout to simulate network issues
      });
      
      // If request succeeds despite short timeout, that's fine
      expect(response.status).toBe(200);
    } catch (error: any) {
      // If request fails due to timeout, that's expected behavior
      expect(error.code === 'ECONNABORTED' || error.message.includes('timeout'));
    }
  }, TEST_TIMEOUT);

  test('should handle high load leading to resource exhaustion', async () => {
    // Wait for services to be fully ready
    await delay(2000);

    // Send many requests simultaneously to test system limits
    const requestCount = 200;
    const requests = [];

    for (let i = 0; i < requestCount; i++) {
      requests.push(
        axios.post(`${AGGREGATOR_URL}/request`, {
          requester_id: `load_test_client_${i}`,
          entropy_type: 'random_bytes',
          size: 16, // Smaller size to reduce memory pressure
          callback_url: null
        }, {
          timeout: 15000
        }).catch(error => {
          // Capture errors as well as successful responses
          return { error, status: error.response?.status || 500 };
        })
      );
    }

    const responses = await Promise.all(requests);

    // Count successful vs failed requests
    const successfulRequests = responses.filter(resp => 
      !resp.hasOwnProperty('error') && resp.status === 200
    ).length;
    
    const failedRequests = responses.length - successfulRequests;

    console.log(`High load test: ${successfulRequests} successful, ${failedRequests} failed out of ${requestCount} requests`);

    // System should handle at least 80% of requests successfully under high load
    const successRate = successfulRequests / requestCount;
    expect(successRate).toBeGreaterThan(0.80);

    // Verify successful responses have correct structure
    responses
      .filter((resp: any) => !resp.hasOwnProperty('error'))
      .forEach((response: any) => {
        if ('status' in response && 'data' in response) {
          expect(response.status).toBe(200);
          expect(response.data).toHaveProperty('request_id');
          expect(response.data).toHaveProperty('entropy');
        }
      });
  }, TEST_TIMEOUT);

  test('should handle malformed JSON requests', async () => {
    // Wait for services to be fully ready
    await delay(2000);

    // Try to send malformed JSON
    try {
      const response = await axios.post(`${AGGREGATOR_URL}/request`, 
        "this is not json",
        {
          headers: {
            'Content-Type': 'application/json'
          },
          timeout: 5000
        }
      );
      
      // Should return an error status for malformed JSON
      expect(response.status).toBeGreaterThanOrEqual(400);
    } catch (error: any) {
      expect(error.response.status).toBeGreaterThanOrEqual(400);
    }

    // Try with incomplete JSON object
    try {
      const response = await axios.post(`${AGGREGATOR_URL}/request`, 
        { requester_id: 'malformed_json_client' }, // Missing required fields
        {
          timeout: 5000
        }
      );
      
      expect(response.status).toBeGreaterThanOrEqual(400);
    } catch (error: any) {
      expect(error.response.status).toBeGreaterThanOrEqual(400);
    }
  }, TEST_TIMEOUT);
});