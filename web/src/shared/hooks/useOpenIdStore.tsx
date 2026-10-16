import { create } from 'zustand';

type Store = StoreValues;

type StoreValues = {
  error?: string;
  details?: string;
};

const defaults: StoreValues = {
  error: undefined,
  details: undefined,
};

export const useOpenidStore = create<Store>(() => ({
  ...defaults,
}));
