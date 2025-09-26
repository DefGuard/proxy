import { Fragment, useEffect } from 'react';
import { MenuItem } from './components/MenuItem';
import './style.scss';
import {
  autoUpdate,
  FloatingPortal,
  offset,
  size,
  useDismiss,
  useFloating,
  useInteractions,
} from '@floating-ui/react';
import { isPresent } from '../../utils/isPresent';
import { MenuHeader } from './components/MenuHeader';
import { MenuSpacer } from './components/MenuSpacer';

import type { MenuProps } from './types';

export const Menu = ({
  itemGroups,
  isOpen,
  referenceRef,
  setOpen,
  placement,
  floatingOffset = 5,
}: MenuProps) => {
  const { refs, floatingStyles, context } = useFloating({
    open: isOpen,
    onOpenChange: setOpen,
    whileElementsMounted: autoUpdate,
    placement: placement ?? 'bottom-start',
    middleware: [
      offset(floatingOffset),
      size({
        apply({ rects, elements }) {
          const w = `${rects.reference.width}px`;
          (elements.floating as HTMLElement).style.minWidth = w;
        },
      }),
    ],
  });

  const dismiss = useDismiss(context, {
    ancestorScroll: true,
    outsidePress: (event) => !(event.target as HTMLElement | null)?.closest('.menu'),
  });

  const { getFloatingProps } = useInteractions([dismiss]);

  useEffect(() => {
    if (referenceRef) {
      refs.setReference(referenceRef.current);
    }
  }, [referenceRef, refs.setReference]);

  if (!isOpen) return null;

  return (
    <FloatingPortal>
      <div
        className="menu"
        {...getFloatingProps()}
        ref={refs.setFloating}
        style={{
          ...floatingStyles,
        }}
      >
        {itemGroups.map((group, groupIndex) => (
          <Fragment key={group.header?.text ?? groupIndex}>
            {isPresent(group.header) && <MenuHeader {...group.header} />}
            {group.items.map((item) => (
              <MenuItem key={item.text} {...item} />
            ))}
            {groupIndex !== 0 && groupIndex !== itemGroups.length - 1 && <MenuSpacer />}
          </Fragment>
        ))}
      </div>
    </FloatingPortal>
  );
};
