import { type CSSProperties, type Ref, useMemo } from 'react';
import type { IconKindValue } from './icon-types';
import './style.scss';
import type { Direction } from '../../types';
import { IconAndroid } from './icons/IconAndroid';
import { IconApple } from './icons/IconApple';
import { IconAppStore } from './icons/IconAppstore';
import { IconArrowBig } from './icons/IconArrowBig';
import { IconArrowSmall } from './icons/IconArrowSmall';
import { IconCheckCircle } from './icons/IconCheckCircle';
import { IconCheckFilled } from './icons/IconCheckFilled';
import { IconClose } from './icons/IconClose';
import { IconDesktop } from './icons/IconDesktop';
import { IconEmptyPoint } from './icons/IconEmptyPoint';
import { IconFile } from './icons/IconFile';
import { IconGlobe } from './icons/IconGlobe';
import { IconLinux } from './icons/IconLinux';
import { IconLoader } from './icons/IconLoader';
import { IconLockOpen } from './icons/IconLock';
import { IconMobile } from './icons/IconMobile';
import { IconPlus } from './icons/IconPlus';
import { IconStatusSimple } from './icons/IconStatusSimple';
import { IconWindows } from './icons/IconWindows';

type Props<T extends IconKindValue = IconKindValue> = {
  icon: T;
  size?: number;
  rotationDirection?: Direction;
  customRotation?: number;
  ref?: Ref<HTMLDivElement>;
};

type RotationMap = Record<Direction, number>;

const mapRotation = (kind: IconKindValue, direction: Direction): number => {
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
export const Icon = <T extends IconKindValue>({
  icon: iconKind,
  rotationDirection,
  customRotation,
  ref,
  size = 20,
}: Props<T>) => {
  const IconToRender = useMemo(() => {
    switch (iconKind) {
      case 'arrow-big':
        return IconArrowBig;
      case 'arrow-small':
        return IconArrowSmall;
      case 'loader':
        return IconLoader;
      case 'plus':
        return IconPlus;
      case 'status-simple':
        return IconStatusSimple;
      case 'lock-open':
        return IconLockOpen;
      case 'check-circle':
        return IconCheckCircle;
      case 'check-filled':
        return IconCheckFilled;
      case 'empty-point':
        return IconEmptyPoint;
      case 'desktop':
        return IconDesktop;
      case 'mobile':
        return IconMobile;
      case 'windows':
        return IconWindows;
      case 'linux':
        return IconLinux;
      case 'app-store':
        return IconAppStore;
      case 'apple':
        return IconApple;
      case 'android':
        return IconAndroid;
      case 'close':
        return IconClose;
      case 'file':
        return IconFile;
      case 'globe':
        return IconGlobe;
      default:
        throw Error(`Unimplemented icon kind: ${iconKind}`);
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
