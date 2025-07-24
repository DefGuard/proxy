import { createWithEqualityFn } from 'zustand/traditional';
import type { AdminInfo, UserInfo } from '../../../shared/hooks/api/types';

const defaultValues: StoreValues = {
  loading: false,
  step: 0,
  sessionEnd: undefined,
  userInfo: undefined,
};

export const usePasswordResetStore = createWithEqualityFn<Store>((set) => ({
  ...defaultValues,
  setState: (values) => set((old) => ({ ...old, ...values })),
  nextStep: (step) => set({ step }),
  reset: () => set(defaultValues),
}));

type Store = StoreValues & StoreMethods;

type StoreValues = {
  loading: boolean;
  step: number;
  sessionStart?: string;
  sessionEnd?: string;
  userInfo?: UserInfo;
  adminInfo?: AdminInfo;
};

type StoreMethods = {
  setState: (values: Partial<StoreValues>) => void;
  nextStep: (step: number) => void;
  reset: () => void;
};
