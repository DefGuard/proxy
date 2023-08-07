import { create } from 'zustand';

const defaultValues: StoreValues = {
  step: 0,
};

export const usePasswordResetStore = create<Store>((set) => ({
  ...defaultValues,
  setState: (values) => set((old) => ({ ...old, ...values })),
  nextStep: (step) => set({ step }),
  reset: () => set(defaultValues),
}));

type Store = StoreValues & StoreMethods;

type StoreValues = {
  step: number;
};

type StoreMethods = {
  setState: (values: Partial<StoreValues>) => void;
  nextStep: (step: number) => void;
  reset: () => void;
};
