import './style.scss';

import classNames from 'classnames';
import { HTMLProps, useMemo } from 'react';

export interface CheckBoxProps
  extends Omit<HTMLProps<HTMLDivElement>, 'onChange' | 'value'> {
  value: boolean;
  disabled?: boolean;
  onChange?: (value: boolean) => void;
}

export const CheckBox = ({
  value,
  onChange,
  disabled = false,
  ...rest
}: CheckBoxProps) => {
  const checked = useMemo(() => (Number(value) ? true : false), [value]);

  const cn = useMemo(
    () =>
      classNames('checkbox', {
        checked: checked,
        disabled: disabled,
      }),
    [checked, disabled],
  );

  return (
    <div
      {...rest}
      className={cn}
      onClick={() => {
        if (onChange && !disabled) {
          onChange(!value);
        }
      }}
    >
      <div className="box"></div>
    </div>
  );
};
