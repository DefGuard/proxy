import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

const defaultValues: StoreValues = {
  sessionEnds: undefined,
};

export const useAppState = create<Store>()(
  devtools((set) => ({
    ...defaultValues,
    setState: (newValues) => set((old) => ({ ...old, ...newValues })),
  })),
);

type Store = StoreValues & StoreMethods;

type StoreValues = {
  sessionEnds?: string;
};

type StoreMethods = {
  setState: (values: Partial<StoreValues>) => void;
};
