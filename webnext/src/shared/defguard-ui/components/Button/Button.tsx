import './style.scss';
import { clsx } from 'clsx';
import { AnimatePresence, motion } from 'motion/react';
import { motionTransitionStandard } from '../../../consts';
import { isPresent } from '../../utils/isPresent';
import { Icon } from '../Icon';
import { LoaderSpinner } from '../LoaderSpinner/LoaderSpinner';
import type { ButtonProps } from './types';

export const Button = ({
  text,
  testId,
  iconLeft,
  iconRight,
  onClick,
  ref,
  iconRightRotation,
  size = 'primary',
  variant = 'primary',
  type = 'button',
  disabled = false,
  loading = false,
  ...props
}: ButtonProps) => {
  return (
    <button
      {...props}
      data-variant={variant}
      data-testid={testId}
      ref={ref}
      type={type}
      disabled={disabled || loading}
      onClick={(e) => {
        if (!disabled && !loading) {
          onClick?.(e);
        }
      }}
      className={clsx('btn', `variant-${variant}`, `size-${size}`, {
        disabled,
        loading: !disabled && loading,
        'icon-left': isPresent(iconLeft) && !isPresent(iconRight),
        'icon-right': isPresent(iconRight) && !isPresent(iconLeft),
        'icon-both': isPresent(iconLeft) && isPresent(iconRight),
      })}
    >
      {isPresent(iconLeft) && <Icon icon={iconLeft} size={20} />}
      <span className="text">{text}</span>
      {isPresent(iconRight) && (
        <Icon icon={iconRight} size={20} rotationDirection={iconRightRotation} />
      )}
      <AnimatePresence mode="wait">
        {loading && !disabled && (
          <motion.div
            className="loader-overlay"
            initial={{
              opacity: 0,
            }}
            animate={{
              opacity: 1,
            }}
            exit={{ opacity: 0 }}
            transition={motionTransitionStandard}
          >
            <LoaderSpinner />
          </motion.div>
        )}
      </AnimatePresence>
    </button>
  );
};
