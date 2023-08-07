import './style.scss';

import classNames from 'classnames';
import { HTMLMotionProps, motion, TargetAndTransition } from 'framer-motion';
import { useMemo, useState } from 'react';

import SvgIconCopy from '../../svg/IconCopy';
import SvgIconDownload from '../../svg/IconDownload';
import SvgIconQr from '../../svg/IconQr';
import { useTheme } from '../hooks/theme/useTheme';
import { ActionButtonVariant } from './types';

type Props = HTMLMotionProps<'button'> & {
  variant: ActionButtonVariant;
  disabled?: boolean;
  className?: string;
  active?: boolean;
  onClick?: () => void;
};

/**
 * Styled button holding icon, created for usage with ExpandableCard and RowBox
 * **/
export const ActionButton = ({
  variant,
  className,
  onClick,
  disabled = false,
  active = false,
  ...rest
}: Props) => {
  const { colors } = useTheme();

  const getIcon = useMemo(() => {
    switch (variant) {
      case ActionButtonVariant.COPY:
        return <SvgIconCopy />;
      case ActionButtonVariant.QRCODE:
        return <SvgIconQr />;
      case ActionButtonVariant.DOWNLOAD:
        return <SvgIconDownload />;
    }
  }, [variant]);

  const cn = useMemo(
    () =>
      classNames(
        'action-button',
        className,
        {
          disabled,
          active,
        },
        `variant-${variant.valueOf()}`,
      ),
    [className, disabled, variant, active],
  );

  const [hovered, setHovered] = useState(false);

  const getAnimate = useMemo((): TargetAndTransition => {
    const res: TargetAndTransition = {
      backgroundColor: colors.surfaceButton,
      opacity: 1,
      transition: {
        duration: 0.25,
      },
    };

    if (disabled) {
      res.opacity = 0.5;
      return res;
    }

    if (hovered || active) {
      res.backgroundColor = colors.surfaceMainPrimary;
    }

    return res;
  }, [colors.surfaceButton, colors.surfaceMainPrimary, disabled, hovered, active]);

  return (
    <motion.button
      {...rest}
      onHoverStart={() => setHovered(true)}
      onHoverEnd={() => setHovered(false)}
      className={cn}
      disabled={disabled}
      initial={false}
      animate={getAnimate}
      onClick={() => {
        if (!disabled && onClick) {
          onClick();
        }
      }}
    >
      {getIcon}
    </motion.button>
  );
};
