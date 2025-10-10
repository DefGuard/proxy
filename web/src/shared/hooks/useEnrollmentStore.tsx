import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import type { EnrollmentStartResponse } from '../api/types';

type Store = Values & Methods;

type Values = {
  token?: string;
  enrollmentData?: EnrollmentStartResponse;
};

type Methods = {
  reset: () => void;
  setState: (values: Partial<Values>) => void;
};

const defaults: Values = {
  enrollmentData: undefined,
  token: undefined,
};

export const useEnrollmentStore = create<Store>()(
  persist(
    (set) => ({
      ...defaults,
      reset: () => set(defaults),
      setState: (values) => set((s) => ({ ...s, ...values })),
    }),
    {
      name: 'enrollment-store',
      version: 1,
      storage: createJSONStorage(() => sessionStorage),
    },
  ),
);
