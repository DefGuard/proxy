import { create } from 'zustand';

type Store = StoreValues;

type StoreValues = {
  error?: string;
};

const defaults: StoreValues = {
  error: undefined,
};

export const useOpenidStore = create<Store>(() => ({
  ...defaults,
}));
