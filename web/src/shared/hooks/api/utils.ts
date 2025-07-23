// biome-ignore lint/suspicious/noExplicitAny: Doesn't care about the type
export const removeNulls = (obj: any) => {
  return JSON.parse(JSON.stringify(obj), (_, value) => {
    if (value == null) return undefined;
    return value;
  });
};
