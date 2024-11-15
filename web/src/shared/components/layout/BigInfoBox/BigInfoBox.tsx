import './style.scss';

import { isUndefined } from 'lodash-es';
import { HTMLProps, ReactNode, useMemo } from 'react';

import SvgIconInfo from '../../svg/IconInfo';

interface Props extends HTMLProps<HTMLDivElement> {
  message?: string | ReactNode;
  children?: ReactNode;
}

/**
 * Big infobox with a message.
 */
export const BigInfoBox = ({ message, children, ...props }: Props) => {
  const renderMessage = useMemo(() => {
    if (!isUndefined(children)) {
      return children;
    }
    if (typeof message === 'string') {
      return <p>{message}</p>;
    }
    return message;
  }, [message, children]);

  return (
    <div className="big-info-box" {...props}>
      <div className="icon-container">
        <SvgIconInfo />
      </div>
      <div className="message">{renderMessage}</div>
    </div>
  );
};
