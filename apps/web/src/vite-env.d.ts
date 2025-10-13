/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE?: string;
  readonly VITE_DEFAULT_MODEL?: string;
  readonly VITE_GITHUB_REDIRECT_PATH?: string;
  readonly VITE_ENABLE_DEV_LOGIN?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
