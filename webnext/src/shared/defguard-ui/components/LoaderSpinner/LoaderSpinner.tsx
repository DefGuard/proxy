import { Icon } from '../Icon';
import './style.scss';

type Props = {
  size?: number;
};

export const LoaderSpinner = ({ size = 20 }: Props) => {
  return (
    <div className="loader-spinner">
      <Icon icon="loader" size={size} />
    </div>
  );
};
