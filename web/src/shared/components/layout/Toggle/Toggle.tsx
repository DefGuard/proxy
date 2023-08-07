import './style.scss';

import classNames from 'classnames';
import equal from 'fast-deep-equal';
import { motion } from 'framer-motion';
import { isUndefined } from 'lodash-es';
import { useMemo } from 'react';

import { ToggleOption } from './components/ToggleOption/ToggleOption';
import { ToggleProps } from './types';

export const Toggle = <T,>({
  selected,
  options,
  onChange,
  disabled = false,
}: ToggleProps<T>) => {
  const activeOptions = useMemo((): number[] => {
    const checkEqual = (first: T, second: T): boolean => {
      if (typeof first == 'object' || Array.isArray(first)) {
        return equal(first, second);
      } else {
        return first === second;
      }
    };
    if (Array.isArray(selected)) {
      return options
        .map((optionItem, index) => {
          if (
            !isUndefined(
              selected.find((selectedItem) => checkEqual(selectedItem, optionItem.value)),
            )
          ) {
            return index;
          }
          return undefined;
        })
        .filter((index) => !isUndefined(index)) as number[];
    } else {
      return [options.findIndex((option) => checkEqual(option.value, selected))];
    }
  }, [options, selected]);

  const cn = useMemo(
    () =>
      classNames('toggle', {
        disabled,
      }),
    [disabled],
  );
  return (
    <motion.div className={cn}>
      {options.map((o, index) => (
        <ToggleOption
          {...o}
          key={index}
          active={activeOptions.includes(index)}
          onClick={() => onChange(o.value)}
        />
      ))}
    </motion.div>
  );
};
