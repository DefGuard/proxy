import './style.scss';

import classNames from 'classnames';
import { type HTMLMotionProps, motion } from 'motion/react';
import { forwardRef, useMemo } from 'react';

export const FloatingBox = forwardRef<HTMLDivElement, HTMLMotionProps<'div'>>(
  ({ className, ...rest }, ref) => {
    const cn = useMemo(() => classNames('floating-box', className), [className]);
    return <motion.div className={cn} ref={ref} {...rest}></motion.div>;
  },
);
