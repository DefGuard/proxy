import './style.scss';

import classNames from 'classnames';
import { motion, TargetAndTransition } from 'framer-motion';
import { useMemo, useState } from 'react';

import { CheckBox } from '../../../Checkbox/CheckBox';
import { useTheme } from '../../../hooks/theme/useTheme';
import { ToggleOptionProps } from '../../types';

export const ToggleOption = <T,>({
  text,
  onClick,
  active,
  disabled = false,
}: ToggleOptionProps<T>) => {
  const { colors } = useTheme();

  const [hovered, setHovered] = useState(false);

  const cn = useMemo(
    () =>
      classNames('toggle-option', {
        active,
        disabled,
      }),
    [active, disabled],
  );

  const getAnimate = useMemo((): TargetAndTransition => {
    const res: TargetAndTransition = {
      opacity: 1,
      color: colors.textBodyTertiary,
      borderColor: colors.borderPrimary,
    };

    if (disabled) {
      res.opacity = 0.5;
    }

    if (active) {
      res.color = colors.textBodyPrimary;
      res.borderColor = colors.surfaceMainPrimary;
    }

    if (hovered && !active) {
      res.borderColor = colors.borderSecondary;
      res.color = colors.textBodyPrimary;
    }

    return res;
  }, [
    active,
    colors.borderPrimary,
    colors.borderSecondary,
    colors.surfaceMainPrimary,
    colors.textBodyPrimary,
    colors.textBodyTertiary,
    disabled,
    hovered,
  ]);

  return (
    <motion.button
      className={cn}
      onClick={() => onClick()}
      disabled={disabled}
      initial={false}
      type="button"
      animate={getAnimate}
      onMouseOver={() => setHovered(true)}
      onMouseOut={() => setHovered(false)}
    >
      <CheckBox value={active} />
      <span>{text}</span>
    </motion.button>
  );
};
