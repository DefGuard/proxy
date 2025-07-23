import { useEffect, useRef } from 'react';

// biome-ignore lint/suspicious/noExplicitAny: Doesn't care about the type
export const removeNulls = (obj: any) => {
  return JSON.parse(JSON.stringify(obj), (_, value) => {
    if (value == null) return undefined;
    return value;
  });
};

/** 
Under normal circumstances, useEffect should run only once when passed an empty dependency array.
However, in dev mode with react strict mode enabled, everything is rendered twice for debugging purposes.
This also causes useEffect to run twice, which is not always desirable.
This custom hook ensures that the effect runs only once in dev mode as well.
*/

// biome-ignore lint/suspicious/noConfusingVoidType: Should work like this
export default function useEffectOnce(fn: () => void | (() => void)) {
  const isMounted = useRef(false);
  useEffect(() => {
    if (isMounted.current) {
      return;
    }

    const callback = fn();
    isMounted.current = true;

    return callback;
  }, [fn]);
}
