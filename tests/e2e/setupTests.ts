// Setup file for Jest tests
// This file is run once before all tests

// Set up environment variables for testing
process.env.ALEA_TEST_MODE = 'true';
process.env.ALEA_LOG_LEVEL = 'info';
process.env.ALEA_NETWORK_MODE = 'testing';
process.env.SGX_MODE = 'SW';
process.env.ALEA_USE_MOCK_SGX = 'true';
process.env.ALEA_TEST_AGGREGATOR_PORT = '8080';
process.env.ALEA_TEST_WORKER_PORT = '8081';

console.log('Test environment configured');