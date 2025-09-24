import './style.scss';
import { useHover } from '@uidotdev/usehooks';
import clsx from 'clsx';
import { useMemo } from 'react';
import { isPresent } from '../../utils/isPresent';

type Props = {
  text?: string;
  active?: boolean;
  disabled?: boolean;
  onClick?: () => void;
};

export const Radio = ({ text, active, disabled, onClick }: Props) => {
  const [ref, hover] = useHover();

  const RenderIcon = useMemo(() => {
    if (active) {
      if (disabled) {
        return StateSelectedDisabled;
      }
      return StateSelected;
    }
    if (disabled) {
      return StateDefaultDisabled;
    }
    if (hover) {
      return StateDefaultHover;
    }
    return StateDefault;
  }, [active, disabled, hover]);

  return (
    <div
      ref={ref}
      onClick={onClick}
      data-active={active}
      data-disabled={disabled}
      role="button"
      aria-disabled={disabled}
      tabIndex={disabled ? -1 : 0}
      className={clsx('radio', {
        text: isPresent(text),
        disabled,
      })}
    >
      <RenderIcon />
      {isPresent(text) && <span>{text}</span>}
    </div>
  );
};

const StateDefault = () => {
  return (
    <svg
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className="icon-default"
    >
      <circle cx="10" cy="10" r="9.5" fill="white" stroke="#DFE3E9" />
    </svg>
  );
};

const StateDefaultHover = () => {
  return (
    <svg
      className="icon-hover"
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <circle cx="10" cy="10" r="9.5" fill="white" stroke="#939CA9" />
    </svg>
  );
};

const StateDefaultDisabled = () => {
  return (
    <svg
      className="icon-disabled"
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <circle cx="10" cy="10" r="9.5" fill="#F7F8FA" stroke="#DFE3E9" />
    </svg>
  );
};

const StateSelected = () => {
  return (
    <svg
      className="icon-selected"
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <circle cx="10" cy="10" r="10" fill="#3961DB" />
      <circle cx="10" cy="10" r="4" fill="white" />
    </svg>
  );
};

const StateSelectedDisabled = () => {
  return (
    <svg
      className="icon-selected-disabled"
      width="20"
      height="20"
      viewBox="0 0 20 20"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <circle cx="10" cy="10" r="10" fill="#B8C0CD" />
      <circle cx="10" cy="10" r="4" fill="#F7F8FA" />
    </svg>
  );
};
