export interface ToggleOption<T> {
  text: string;
  disabled?: boolean;
  value: T;
}

export interface ToggleOptionProps<T> extends ToggleOption<T> {
  onClick: () => void;
  active: boolean;
}

export interface ToggleProps<T> {
  selected: T | T[];
  options: ToggleOption<T>[];
  onChange: (v: T) => void;
  disabled?: boolean;
}
