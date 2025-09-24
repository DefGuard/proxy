import './style.scss';
import clsx from 'clsx';
import type { OrientationValue } from '../../types';
import { isPresent } from '../../utils/isPresent';

type Props = {
  text?: string;
  orientation?: OrientationValue;
};

export const Divider = ({ text, orientation }: Props) => {
  const textPresent = isPresent(text) && text.length > 0;

  return (
    <div
      className={clsx('divider', orientation, {
        text: textPresent,
      })}
    >
      {orientation === 'horizontal' && (
        <>
          {textPresent && (
            <>
              <Line />
              <span>{text}</span>
              <Line />
            </>
          )}
          {!textPresent && <Line />}
        </>
      )}
      {orientation === 'vertical' && <Line />}
    </div>
  );
};

const Line = () => {
  return <div className="line"></div>;
};
