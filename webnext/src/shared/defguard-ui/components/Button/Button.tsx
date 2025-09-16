import { clsx } from 'clsx';

type ButtonVariant = '' | '';

type Props = {
  text: string;
  variant: ButtonVariant;
};

export const Button = ({}: Props) => {
  return (
    <button className={clsx('btn')}>
      <span className="text"></span>
      <span className="loader">loader_placeholder</span>
    </button>
  );
};
