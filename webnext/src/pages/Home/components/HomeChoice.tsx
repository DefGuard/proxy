import './style.scss';

import clsx from 'clsx';
import { m } from '../../../paraglide/messages';
import { Button } from '../../../shared/defguard-ui/components/Button/Button';
import type { IconKindValue } from '../../../shared/defguard-ui/components/Icon/icon-types';
import { SizedBox } from '../../../shared/defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../../shared/defguard-ui/types';

export const HomeChoice = () => {
  return (
    <div id="home-choice">
      <Card
        title={m.start_multi_enrollment_title()}
        subtitle={m.start_multi_enrollment_subtitle()}
        buttonText={m.start_multi_enrollment_button()}
        buttonIcon="lock-open"
        onClick={() => {}}
      />
      <Card
        title={m.start_multi_password_title()}
        subtitle={m.start_multi_password_subtitle()}
        buttonText={m.start_multi_password_button()}
        buttonIcon="lock-open"
        onClick={() => {}}
      />
    </div>
  );
};

type CardProps = {
  buttonIcon: IconKindValue;
  buttonText: string;
  subtitle: string;
  title: string;
  card?: boolean;
  onClick: () => void;
};

const Card = ({ buttonIcon, buttonText, subtitle, title, onClick }: CardProps) => {
  return (
    <div className={clsx('choice')}>
      <div className="image"></div>
      <SizedBox height={37} />
      <p className="title">{title}</p>
      <SizedBox height={ThemeSpacing.Md} />
      <p className="subtitle">{subtitle}</p>
      <SizedBox height={ThemeSpacing.Xl2} />
      <Button
        size="primary"
        variant="primary"
        text={buttonText}
        iconLeft={buttonIcon}
        onClick={onClick}
      />
    </div>
  );
};
