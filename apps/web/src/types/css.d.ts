import "solid-js";

declare global {
  namespace JSX {
    interface CSSProperties {
      [key: string]: any;
    }
  }
}
