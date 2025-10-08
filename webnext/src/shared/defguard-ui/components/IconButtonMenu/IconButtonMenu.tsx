import {
  autoUpdate,
  FloatingPortal,
  offset,
  shift,
  size,
  useClick,
  useDismiss,
  useFloating,
  useInteractions,
} from '@floating-ui/react';
import { useState } from 'react';
import { mergeRefs } from '../../utils/mergeRefs';
import { IconButton } from '../IconButton/IconButton';
import type { IconButtonProps } from '../IconButton/types';
import { Menu } from '../Menu/Menu';
import type { MenuItemsGroup } from '../Menu/types';

export const IconButtonMenu = ({
  menuItems,
  ref,
  ...buttonProps
}: IconButtonProps & {
  menuItems: MenuItemsGroup[];
}) => {
  const [isOpen, setOpen] = useState(false);
  const { refs, context, floatingStyles } = useFloating({
    placement: 'bottom-end',
    whileElementsMounted: autoUpdate,
    onOpenChange: setOpen,
    open: isOpen,
    middleware: [
      offset(4),
      shift(),
      size({
        apply({ rects, elements, availableHeight }) {
          const refWidth = `${rects.reference.width}px`;
          elements.floating.style.minWidth = refWidth;
          elements.floating.style.maxHeight = `${availableHeight - 10}px`;
        },
      }),
    ],
  });
  const click = useClick(context, {
    toggle: true,
  });

  const dismiss = useDismiss(context, {
    ancestorScroll: true,
    escapeKey: true,
    outsidePress: (event) => !(event.target as HTMLElement).closest('.menu'),
  });

  const { getFloatingProps, getReferenceProps } = useInteractions([click, dismiss]);

  return (
    <>
      <IconButton
        {...buttonProps}
        {...getReferenceProps()}
        ref={mergeRefs([ref, refs.setReference])}
      />
      {isOpen && (
        <FloatingPortal>
          <Menu
            itemGroups={menuItems}
            ref={refs.setFloating}
            style={floatingStyles}
            onClose={() => {
              setOpen(false);
            }}
            {...getFloatingProps()}
          />
        </FloatingPortal>
      )}
    </>
  );
};
