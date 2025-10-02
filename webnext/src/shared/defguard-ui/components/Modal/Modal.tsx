import './style.scss';
import clsx from 'clsx';
import { AnimatePresence, motion } from 'motion/react';
import { useCallback, useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { BehaviorSubject } from 'rxjs';
import { motionTransitionStandard } from '../../../consts';
import { isPresent } from '../../utils/isPresent';
import { IconButton } from '../IconButton/IconButton';
import type { ModalProps } from './types';

const portalTarget = document.getElementById('modals-root') as HTMLElement;
const rootElement = document.getElementById('root') as HTMLElement;

type MouseObserverState = {
  press?: React.MouseEvent<HTMLDivElement, MouseEvent>;
  release?: React.MouseEvent<HTMLDivElement, MouseEvent>;
};

export const Modal = ({
  id,
  isOpen,
  contentClassName,
  positionerClassName,
  hideBackdrop,
  title,
  children,
  size,
  afterClose,
  onClose,
}: ModalProps) => {
  const openRef = useRef(isOpen);
  const contentRef = useRef<HTMLDivElement | null>(null);
  const mouseObserver = useRef(new BehaviorSubject<MouseObserverState>({}));

  const checkEventOutside = useCallback(
    (event: React.MouseEvent<HTMLDivElement, MouseEvent>): boolean => {
      const domRect = contentRef.current?.getBoundingClientRect();
      if (domRect) {
        const start_x = domRect?.x;
        const start_y = domRect?.y;
        const end_x = start_x + domRect?.width;
        const end_y = start_y + domRect.height;
        if (
          event.clientX < start_x ||
          event.clientX > end_x ||
          event.clientY < start_y ||
          event.clientY > end_y
        ) {
          return true;
        }
      }
      return false;
    },
    [],
  );

  useEffect(() => {
    if (mouseObserver && contentRef && isOpen) {
      const sub = mouseObserver.current.subscribe(({ press, release }) => {
        if (release && press) {
          const target = press.target as Element;
          const validParent = target.closest('#modals-root');
          const checkPress = checkEventOutside(press);
          const checkRelease = checkEventOutside(release);
          if (checkPress && checkRelease && isPresent(onClose) && validParent !== null) {
            onClose();
          }
        }
      });
      return () => {
        sub.unsubscribe();
      };
    }
  }, [isOpen, onClose, checkEventOutside]);

  useEffect(() => {
    // clear observer after closing modal
    if (!isOpen) {
      mouseObserver.current.next({});
    }
    if (isOpen) {
      rootElement.setAttribute('aria-hidden', 'true');
      rootElement.style.overflowY = 'hidden';
    } else {
      rootElement.removeAttribute('aria-hidden');
      rootElement.style.overflowY = 'auto';
    }
  }, [isOpen]);

  return createPortal(
    <AnimatePresence mode="wait">
      {isOpen && (
        <motion.div className="modal-root">
          {!hideBackdrop && (
            <motion.div
              className="backdrop"
              style={{
                backgroundColor: '#000000',
              }}
              initial={{
                opacity: 0,
              }}
              animate={{
                opacity: 0.45,
              }}
              exit={{
                opacity: 0,
              }}
              transition={motionTransitionStandard}
            ></motion.div>
          )}
          <motion.div
            className={clsx('modal-positioner', positionerClassName)}
            onMouseUp={(event) => {
              if (event) {
                const { press } = mouseObserver.current.getValue();
                mouseObserver.current.next({ press: press, release: event });
              }
            }}
            onMouseDown={(event) => {
              if (event) {
                mouseObserver.current.next({ press: event, release: undefined });
              }
            }}
          >
            <motion.div
              id={id}
              ref={contentRef}
              className={clsx('modal', contentClassName, `size-${size}`)}
              initial={{
                opacity: 0,
              }}
              animate={{
                opacity: 1,
              }}
              exit={{
                opacity: 0,
              }}
              onAnimationComplete={() => {
                if (!openRef.current) {
                  afterClose?.();
                }
              }}
              transition={motionTransitionStandard}
            >
              <div className="modal-header">
                <p className="title">{title}</p>
                <IconButton
                  icon="close"
                  onClick={onClose}
                  disabled={!isPresent(onClose)}
                />
              </div>
              <div className="modal-content">{children}</div>
            </motion.div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>,
    portalTarget,
  );
};
