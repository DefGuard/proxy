import { useCallback } from 'react';

export const useClipboard = () => {
  const writeToClipboard = useCallback(async (value: string) => {
    if (window.isSecureContext) {
      try {
        await navigator.clipboard.writeText(value);
      } catch (e) {
        console.error(e);
      }
    } else {
      console.warn('Cannot access clipboard in insecure contexts');
    }
  }, []);
  return {
    writeToClipboard,
  };
};
