/// <reference types="vite/client" />

declare module '@vitejs/plugin-react' {
  import { Plugin } from 'vite';
  function react(): Plugin;
  export default react;
}