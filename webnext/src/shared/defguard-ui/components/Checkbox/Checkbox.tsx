import clsx from 'clsx';
import './style.scss';
import { useHover } from '@uidotdev/usehooks';
import { useId, useMemo } from 'react';
import { isPresent } from '../../utils/isPresent';

type Props = {
  canError?: boolean;
  active?: boolean;
  error?: string;
  disabled?: boolean;
  text?: string;
  onClick?: () => void;
};

// todo: Include error text when it will be needed
export const Checkbox = ({
  text,
  error,
  canError = false,
  active = false,
  disabled = false,
  onClick,
}: Props) => {
  const hasError = isPresent(error);

  const [ref, hover] = useHover();

  const ContentRender = useMemo(() => {
    if (disabled) {
      if (!active) return StateDefaultDisabled;
      return StateSelectedDisabled;
    }
    if (active) {
      return StateSelected;
    }
    if (hasError) {
      return StateError;
    }
    if (hover) {
      return StateHover;
    }
    return StateDefault;
  }, [hover, active, hasError, disabled]);

  return (
    <div
      className={clsx('checkbox', {
        error: hasError,
        'can-error': canError,
        text: isPresent(text),
      })}
      onClick={onClick}
      ref={ref}
      role="button"
      tabIndex={disabled ? -1 : 0}
      data-active={active}
    >
      <ContentRender />
      {isPresent(text) && <span>{text}</span>}
    </div>
  );
};

const StateDefault = () => {
  const id = useId();
  return (
    <svg
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <mask id={id} style={{ fill: 'var(--bg-default)' }}>
        <path d="M2 6C2 3.79086 3.79086 2 6 2H18C20.2091 2 22 3.79086 22 6V18C22 20.2091 20.2091 22 18 22H6C3.79086 22 2 20.2091 2 18V6Z" />
      </mask>
      <path
        d="M2 6C2 3.79086 3.79086 2 6 2H18C20.2091 2 22 3.79086 22 6V18C22 20.2091 20.2091 22 18 22H6C3.79086 22 2 20.2091 2 18V6Z"
        style={{ fill: 'var(--bg-default)' }}
      />
      <path
        d="M6 2V3H18V2V1H6V2ZM22 6H21V18H22H23V6H22ZM18 22V21H6V22V23H18V22ZM2 18H3V6H2H1V18H2ZM6 22V21C4.34315 21 3 19.6569 3 18H2H1C1 20.7614 3.23858 23 6 23V22ZM22 18H21C21 19.6569 19.6569 21 18 21V22V23C20.7614 23 23 20.7614 23 18H22ZM18 2V3C19.6569 3 21 4.34315 21 6H22H23C23 3.23858 20.7614 1 18 1V2ZM6 2V1C3.23858 1 1 3.23858 1 6H2H3C3 4.34315 4.34315 3 6 3V2Z"
        mask={`url(#${id})`}
        style={{ fill: 'var(--border-default)' }}
      />
    </svg>
  );
};
const StateDefaultDisabled = () => {
  return (
    <svg
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M6 2.5H18C19.933 2.5 21.5 4.067 21.5 6V18C21.5 19.933 19.933 21.5 18 21.5H6C4.067 21.5 2.5 19.933 2.5 18V6C2.5 4.067 4.067 2.5 6 2.5Z"
        fill="#F7F8FA"
      />
      <path
        d="M6 2.5H18C19.933 2.5 21.5 4.067 21.5 6V18C21.5 19.933 19.933 21.5 18 21.5H6C4.067 21.5 2.5 19.933 2.5 18V6C2.5 4.067 4.067 2.5 6 2.5Z"
        stroke="#DFE3E9"
      />
    </svg>
  );
};
const StateHover = () => {
  return (
    <svg
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <mask id="path-1-inside-1_2042_1139" fill="white">
        <path d="M2 6C2 3.79086 3.79086 2 6 2H18C20.2091 2 22 3.79086 22 6V18C22 20.2091 20.2091 22 18 22H6C3.79086 22 2 20.2091 2 18V6Z" />
      </mask>
      <path
        d="M2 6C2 3.79086 3.79086 2 6 2H18C20.2091 2 22 3.79086 22 6V18C22 20.2091 20.2091 22 18 22H6C3.79086 22 2 20.2091 2 18V6Z"
        fill="white"
      />
      <path
        d="M6 2V3H18V2V1H6V2ZM22 6H21V18H22H23V6H22ZM18 22V21H6V22V23H18V22ZM2 18H3V6H2H1V18H2ZM6 22V21C4.34315 21 3 19.6569 3 18H2H1C1 20.7614 3.23858 23 6 23V22ZM22 18H21C21 19.6569 19.6569 21 18 21V22V23C20.7614 23 23 20.7614 23 18H22ZM18 2V3C19.6569 3 21 4.34315 21 6H22H23C23 3.23858 20.7614 1 18 1V2ZM6 2V1C3.23858 1 1 3.23858 1 6H2H3C3 4.34315 4.34315 3 6 3V2Z"
        fill="#939CA9"
        mask="url(#path-1-inside-1_2042_1139)"
      />
    </svg>
  );
};
const StateSelected = () => {
  return (
    <svg
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M2 6C2 3.79086 3.79086 2 6 2H18C20.2091 2 22 3.79086 22 6V18C22 20.2091 20.2091 22 18 22H6C3.79086 22 2 20.2091 2 18V6Z"
        fill="#3961DB"
      />
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M18 9.47029L10.883 16L7 12.4374L8.29805 10.9671L10.883 13.3388L16.7019 8L18 9.47029Z"
        fill="white"
      />
    </svg>
  );
};
const StateSelectedDisabled = () => {
  return (
    <svg
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M2 6C2 3.79086 3.79086 2 6 2H18C20.2091 2 22 3.79086 22 6V18C22 20.2091 20.2091 22 18 22H6C3.79086 22 2 20.2091 2 18V6Z"
        fill="#B8C0CD"
      />
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M18 9.47029L10.883 16L7 12.4374L8.29805 10.9671L10.883 13.3388L16.7019 8L18 9.47029Z"
        fill="#F7F8FA"
      />
    </svg>
  );
};
// const StateMixed = () => {
//   return (
//     <svg
//       width="24"
//       height="24"
//       viewBox="0 0 24 24"
//       fill="none"
//       xmlns="http://www.w3.org/2000/svg"
//     >
//       <path
//         d="M2 6C2 3.79086 3.79086 2 6 2H18C20.2091 2 22 3.79086 22 6V18C22 20.2091 20.2091 22 18 22H6C3.79086 22 2 20.2091 2 18V6Z"
//         fill="#3961DB"
//       />
//       <path
//         fill-rule="evenodd"
//         clip-rule="evenodd"
//         d="M17.5 13H7V11H17.5V13Z"
//         fill="white"
//       />
//     </svg>
//   );
// };
// const StateMixedDisabled = () => {
//   return (
//     <svg
//       width="24"
//       height="24"
//       viewBox="0 0 24 24"
//       fill="none"
//       xmlns="http://www.w3.org/2000/svg"
//     >
//       <path
//         d="M2 6C2 3.79086 3.79086 2 6 2H18C20.2091 2 22 3.79086 22 6V18C22 20.2091 20.2091 22 18 22H6C3.79086 22 2 20.2091 2 18V6Z"
//         fill="#B8C0CD"
//       />
//       <path
//         fillRule="evenodd"
//         clipRule="evenodd"
//         d="M17.5 13H7V11H17.5V13Z"
//         fill="#F7F8FA"
//       />
//     </svg>
//   );
// };
const StateError = () => {
  return (
    <svg
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <mask id="path-1-inside-1_5474_915" fill="white">
        <path d="M2 6C2 3.79086 3.79086 2 6 2H18C20.2091 2 22 3.79086 22 6V18C22 20.2091 20.2091 22 18 22H6C3.79086 22 2 20.2091 2 18V6Z" />
      </mask>
      <path
        d="M2 6C2 3.79086 3.79086 2 6 2H18C20.2091 2 22 3.79086 22 6V18C22 20.2091 20.2091 22 18 22H6C3.79086 22 2 20.2091 2 18V6Z"
        fill="white"
      />
      <path
        d="M6 2V3H18V2V1H6V2ZM22 6H21V18H22H23V6H22ZM18 22V21H6V22V23H18V22ZM2 18H3V6H2H1V18H2ZM6 22V21C4.34315 21 3 19.6569 3 18H2H1C1 20.7614 3.23858 23 6 23V22ZM22 18H21C21 19.6569 19.6569 21 18 21V22V23C20.7614 23 23 20.7614 23 18H22ZM18 2V3C19.6569 3 21 4.34315 21 6H22H23C23 3.23858 20.7614 1 18 1V2ZM6 2V1C3.23858 1 1 3.23858 1 6H2H3C3 4.34315 4.34315 3 6 3V2Z"
        fill="#CC3C3C"
        mask="url(#path-1-inside-1_5474_915)"
      />
    </svg>
  );
};
