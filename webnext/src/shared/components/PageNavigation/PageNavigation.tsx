import { Button } from '../../defguard-ui/components/Button/Button';
import './style.scss';

type Props = {
  loading?: boolean;
  nextText: string;
  backText: string;
  onNext?: () => void;
  onBack?: () => void;
  backDisabled?: boolean;
  nextDisabled?: boolean;
};

export const PageNavigation = ({
  backText,
  nextText,
  backDisabled,
  nextDisabled,
  onBack,
  onNext,
  loading = false,
}: Props) => {
  return (
    <div className="page-nav">
      <div className="content">
        <div className="track">
          <Button
            text={backText}
            disabled={backDisabled || loading}
            variant="outlined"
            onClick={onBack}
          />
          <Button
            text={nextText}
            disabled={nextDisabled}
            loading={loading}
            onClick={onNext}
          />
        </div>
      </div>
    </div>
  );
};
