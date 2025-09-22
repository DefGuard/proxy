import { Button } from '../../defguard-ui/components/Button/Button';
import './style.scss';

type Props = {
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
}: Props) => {
  return (
    <div className="page-nav">
      <div className="content">
        <div className="track">
          <Button
            text={backText}
            disabled={backDisabled}
            variant="outlined"
            onClick={onBack}
          />
          <Button text={nextText} disabled={nextDisabled} onClick={onNext} />
        </div>
      </div>
    </div>
  );
};
