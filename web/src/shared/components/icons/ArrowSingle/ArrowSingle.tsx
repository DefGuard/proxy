import './style.scss';

import classNames from 'classnames';
import { useMemo } from 'react';

import SvgIconArrowSingleLarge from '../../svg/IconArrowSingleLarge';
import SvgIconArrowSingleSmall from '../../svg/IconArrowSingleSmall';
import { ArrowSingleDirection, ArrowSingleSize } from './types';

type Props = {
  direction?: ArrowSingleDirection;
  size?: ArrowSingleSize;
  className?: string;
};

export const ArrowSingle = ({
  className,
  direction = ArrowSingleDirection.RIGHT,
  size = ArrowSingleSize.SMALL,
}: Props) => {
  const cn = classNames('arrow-single', className);

  const getRotation = useMemo((): number => {
    switch (direction) {
      case ArrowSingleDirection.RIGHT:
        return 0;
      case ArrowSingleDirection.LEFT:
        return 180;
      case ArrowSingleDirection.UP:
        return -90;
      case ArrowSingleDirection.DOWN:
        return 90;
    }
  }, [direction]);

  return (
    <div className={cn}>
      {size === ArrowSingleSize.SMALL && (
        <SvgIconArrowSingleSmall style={{ transform: `rotateZ(${getRotation}deg)` }} />
      )}
      {size === ArrowSingleSize.LARGE && (
        <SvgIconArrowSingleLarge style={{ transform: `rotateZ(${getRotation}deg)` }} />
      )}
    </div>
  );
};
