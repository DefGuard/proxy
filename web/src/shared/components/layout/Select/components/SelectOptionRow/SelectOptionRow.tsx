import classNames from 'classnames';
import { HTMLMotionProps, motion } from 'framer-motion';
import { useMemo, useState } from 'react';

import { useTheme } from '../../../hooks/theme/useTheme';

type Props = HTMLMotionProps<'button'> & {
  label?: string;
  selected?: boolean;
  // Marks said option as the one that make new options
  createOption?: boolean;
};

export const SelectOptionRow = ({
  label,
  selected,
  className,
  createOption,
  ...rest
}: Props) => {
  const { colors } = useTheme();

  const [hovered, setHovered] = useState(false);

  const cn = useMemo(
    () =>
      classNames('select-option', className, {
        hovered,
        selected,
        'create-option': createOption,
      }),
    [className, hovered, selected, createOption],
  );

  const getAnimate = useMemo(() => {
    const res = {
      color: colors.textBodySecondary,
    };

    if (createOption) {
      res.color = colors.surfaceMainPrimary;
      return res;
    }

    if (hovered || selected) {
      res.color = colors.textBodyPrimary;
    }

    return res;
  }, [
    colors.surfaceMainPrimary,
    colors.textBodyPrimary,
    colors.textBodySecondary,
    createOption,
    hovered,
    selected,
  ]);

  return (
    <motion.button
      {...rest}
      type="button"
      className={cn}
      initial={false}
      animate={getAnimate}
      onHoverStart={() => setHovered(true)}
      onHoverEnd={() => setHovered(false)}
    >
      <span>{label}</span>
    </motion.button>
  );
};
