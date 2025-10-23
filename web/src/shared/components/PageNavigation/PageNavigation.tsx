import { Button } from '../../defguard-ui/components/Button/Button';
import { isPresent } from '../../defguard-ui/utils/isPresent';
import './style.scss';

type Props = {
  loading?: boolean;
  nextText: string;
  backText?: string;
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
          {!isPresent(backText) && <div className="element-fill"></div>}
          {isPresent(backText) && (
            <Button
              testId="page-nav-back"
              text={backText}
              disabled={backDisabled || loading}
              variant="outlined"
              onClick={onBack}
            />
          )}
          <Button
            testId="page-nav-next"
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
