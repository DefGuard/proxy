import { type CSSProperties, type Ref, useMemo } from 'react';
import type { IconKind } from './icon-types';
import './style.scss';
import type { Direction } from '../../types';
import { IconArrowBig } from './icons/IconArrowBig';
import { IconArrowSmall } from './icons/IconArrowSmall';

type Props<T extends IconKind = IconKind> = {
  icon: T;
  size?: number;
  rotationDirection?: Direction;
  customRotation?: number;
  ref?: Ref<HTMLDivElement>;
};

type RotationMap = Record<Direction, number>;

const mapRotation = (kind: IconKind, direction: Direction): number => {
  switch (kind) {
    case 'arrow-small':
    case 'arrow-big': {
      const map: RotationMap = {
        down: 90,
        right: 0,
        up: -90,
        left: 180,
      };
      return map[direction];
    }
  }
  console.error(`Unimplemented rotation mapping for icon kind of ${kind}`);
  // safe return for unimplemented
  return 0;
};

// Color should be set by css bcs some icons have different structures like 'loader'
export const Icon = <T extends IconKind>({
  icon: iconKind,
  rotationDirection,
  customRotation,
  ref,
  size,
}: Props<T>) => {
  const IconToRender = useMemo(() => {
    switch (iconKind) {
      case 'arrow-big': {
        return IconArrowBig;
      }
      case 'arrow-small':
        return IconArrowSmall;
      default:
        throw Error('Unimplemented icon kind');
    }
  }, [iconKind]);

  const getStyle = useMemo((): CSSProperties => {
    const styles: CSSProperties = {};
    const transform: string[] = [];
    // kind specific configurations
    switch (iconKind) {
      case 'arrow-big':
      case 'arrow-small':
        if (rotationDirection) {
          transform.push(`rotate(${mapRotation(iconKind, rotationDirection)}deg)`);
        }
        break;
    }
    if (customRotation && !rotationDirection) {
      transform.push(`rotate(${customRotation}deg)`);
    }
    if (size) {
      styles.width = size;
      styles.height = size;
    }
    if (transform.length) {
      styles.transform = transform.join(' ');
    }
    return styles;
  }, [iconKind, size, rotationDirection, customRotation]);

  return (
    <div className="icon" ref={ref} style={getStyle} data-kind={iconKind}>
      <IconToRender />
    </div>
  );
};
