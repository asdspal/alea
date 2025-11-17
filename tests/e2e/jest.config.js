module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testMatch: [
    '**/tests/e2e/**/*.test.ts'
  ],
  collectCoverageFrom: [
    '**/tests/e2e/**/*.{ts,js}',
    '!**/node_modules/**',
    '!**/coverage/**'
  ],
  coverageDirectory: './coverage',
  coverageReporters: ['text', 'lcov', 'html'],
  setupFilesAfterEnv: ['<rootDir>/setupTests.ts'],
  testTimeout: 60000, // 60 seconds timeout for tests
  verbose: true,
  // Reduce output for cleaner test runs
  silent: false,
  globals: {
    'ts-jest': {
      tsconfig: 'tsconfig.json'
    }
  }
};