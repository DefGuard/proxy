import { pick } from 'lodash-es';
import { create } from 'zustand';
import { createJSONStorage, devtools, persist } from 'zustand/middleware';

import { UserInfo } from '../../types';

const defaultValues: StoreValues = {
  step: 0,
  stepsMax: 4,
  sessionEnd: undefined,
  userInfo: undefined,
};

const persistKeys: Array<keyof StoreValues> = [
  'step',
  'userInfo',
  'sessionEnd',
  'userInfo',
];

export const useEnrollmentStore = create<Store>()(
  devtools(
    persist(
      (set, get) => ({
        ...defaultValues,
        init: (values) => set({ ...defaultValues, ...values }),
        setState: (newValues) => set((old) => ({ ...old, ...newValues })),
        reset: () => set(defaultValues),
        nextStep: () => {
          const current = get().step;
          const max = get().stepsMax;

          if (current < max) {
            return set({ step: current + 1 });
          }
        },
        perviousStep: () => {
          const current = get().step;

          if (current > 0) {
            return set({ step: current - 1 });
          }
        },
      }),
      {
        name: 'enrollment-storage',
        version: 0.1,
        storage: createJSONStorage(() => localStorage),
        partialize: (state) => pick(state, persistKeys),
      },
    ),
  ),
);

type Store = StoreValues & StoreMethods;

type StoreValues = {
  step: number;
  stepsMax: number;
  sessionEnd?: string;
  userInfo?: UserInfo;
};

type StoreMethods = {
  setState: (values: Partial<StoreValues>) => void;
  reset: () => void;
  nextStep: () => void;
  perviousStep: () => void;
  init: (initValues: Partial<StoreValues>) => void;
};
