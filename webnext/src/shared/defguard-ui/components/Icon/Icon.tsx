import { type CSSProperties, type Ref, useMemo } from 'react';
import type { IconKindValue } from './icon-types';
import './style.scss';
import type { Direction } from '../../types';
import { IconAccessSettings } from './icons/IconAccessSettings';
import { IconAndroid } from './icons/IconAndroid';
import { IconApple } from './icons/IconApple';
import { IconAppStore } from './icons/IconAppstore';
import { IconArchLinux } from './icons/IconArchLinux';
import { IconArrowBig } from './icons/IconArrowBig';
import { IconArrowSmall } from './icons/IconArrowSmall';
import { IconCheckCircle } from './icons/IconCheckCircle';
import { IconCheckFilled } from './icons/IconCheckFilled';
import { IconClose } from './icons/IconClose';
import { IconConfig } from './icons/IconConfig';
import { IconCopy } from './icons/IconCopy';
import { IconDebian } from './icons/IconDebian';
import { IconDesktop } from './icons/IconDesktop';
import { IconDisabled } from './icons/IconDisabled';
import { IconEmptyPoint } from './icons/IconEmptyPoint';
import { IconFile } from './icons/IconFile';
import { IconGlobe } from './icons/IconGlobe';
import { IconHelp } from './icons/IconHelp';
import { IconHide } from './icons/IconHide';
import { IconLinux } from './icons/IconLinux';
import { IconLoader } from './icons/IconLoader';
import { IconLockOpen } from './icons/IconLock';
import { IconMobile } from './icons/IconMobile';
import { IconOpenInNewWindow } from './icons/IconOpenInNewWindow';
import { IconPlus } from './icons/IconPlus';
import { IconShow } from './icons/IconShow';
import { IconStatusSimple } from './icons/IconStatusSimple';
import { IconUbuntu } from './icons/IconUbuntu';
import { IconWarning } from './icons/IconWarning';
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

const EmptyIcon = () => {
  return null;
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
      case 'warning':
        return IconWarning;
      case 'ubuntu':
        return IconUbuntu;
      case 'debian':
        return IconDebian;
      case 'arch-linux':
        return IconArchLinux;
      case 'disabled':
        return IconDisabled;
      case 'show':
        return IconShow;
      case 'hide':
        return IconHide;
      case 'copy':
        return IconCopy;
      case 'config':
        return IconConfig;
      case 'open-in-new-window':
        return IconOpenInNewWindow;
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
      case 'help':
        return IconHelp;
      case 'access-settings':
        return IconAccessSettings;
      case 'activity':
        return EmptyIcon;
      case 'activity-notes':
        return EmptyIcon;
      case 'add-user':
        return EmptyIcon;
      case 'analytics':
        return EmptyIcon;
      case 'archive':
        return EmptyIcon;
      case 'attention':
        return EmptyIcon;
      case 'check':
        return EmptyIcon;
      case 'clear':
        return EmptyIcon;
      case 'code':
        return EmptyIcon;
      case 'collapse':
        return EmptyIcon;
      case 'credit-card':
        return EmptyIcon;
      case 'date':
        return EmptyIcon;
      case 'delete':
        return EmptyIcon;
      case 'deploy':
        return EmptyIcon;
      case 'devices':
        return EmptyIcon;
      case 'devices-active':
        return EmptyIcon;
      case 'download':
        return EmptyIcon;
      case 'edit':
        return EmptyIcon;
      case 'enter':
        return EmptyIcon;
      case 'expand':
        return EmptyIcon;
      case 'filter':
        return EmptyIcon;
      case 'gateway':
        return EmptyIcon;
      case 'gift':
        return EmptyIcon;
      case 'github':
        return EmptyIcon;
      case 'groups':
        return EmptyIcon;
      case 'hamburger':
        return EmptyIcon;
      case 'info-filled':
        return EmptyIcon;
      case 'info-outlined':
        return EmptyIcon;
      case 'location':
        return EmptyIcon;
      case 'location-preview':
        return EmptyIcon;
      case 'location-tracking':
        return EmptyIcon;
      case 'logout':
        return EmptyIcon;
      case 'mail':
        return EmptyIcon;
      case 'manage-keys':
        return EmptyIcon;
      case 'menu':
        return EmptyIcon;
      case 'minus-circle':
        return EmptyIcon;
      case 'navigation-collapse':
        return EmptyIcon;
      case 'navigation-uncollapse':
        return EmptyIcon;
      case 'notification':
        return EmptyIcon;
      case 'one-time-password':
        return EmptyIcon;
      case 'openid':
        return EmptyIcon;
      case 'pdf':
        return EmptyIcon;
      case 'pie-chart':
        return EmptyIcon;
      case 'plus-circle':
        return EmptyIcon;
      case 'profile':
        return EmptyIcon;
      case 'protection':
        return EmptyIcon;
      case 'qr':
        return EmptyIcon;
      case 'search':
        return EmptyIcon;
      case 'servers':
        return EmptyIcon;
      case 'settings':
        return EmptyIcon;
      case 'sort':
        return EmptyIcon;
      case 'status-attention':
        return EmptyIcon;
      case 'status-available':
        return EmptyIcon;
      case 'status-important':
        return EmptyIcon;
      case 'support':
        return EmptyIcon;
      case 'transactions':
        return EmptyIcon;
      case 'user':
        return EmptyIcon;
      case 'user-active':
        return EmptyIcon;
      case 'users':
        return EmptyIcon;
      case 'webhooks':
        return EmptyIcon;
      case 'yubi-keys':
        return EmptyIcon;
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
