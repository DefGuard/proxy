import './style.scss';

import clsx from 'clsx';
import type { Ref } from 'react';

type Props = {
  ref?: Ref<HTMLLabelElement>;
  htmlFor?: string;
  text: string;
  required?: boolean;
};

export const FieldLabel = ({ text, htmlFor, ref, required }: Props) => {
  return (
    <label
      className={clsx('field-label', {
        required,
      })}
      htmlFor={htmlFor}
      ref={ref}
    >
      {required && (
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="5"
          height="5"
          viewBox="0 0 5 5"
          fill="none"
        >
          <path
            d="M1.332 4.128L0.6 3.684L1.368 2.52H0V1.608L1.368 1.62L0.6 0.456L1.332 0L2.064 1.224L2.808 0L3.528 0.456L2.76 1.62L4.128 1.608V2.52H2.76L3.528 3.684L2.808 4.128L2.064 2.916L1.332 4.128Z"
            fill="#CC3C3C"
          />
        </svg>
      )}
      {text}
    </label>
  );
};
