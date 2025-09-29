import './style.scss';

import { Link } from '@tanstack/react-router';
import clsx from 'clsx';
import { m } from '../../../paraglide/messages';
import { Button } from '../../../shared/defguard-ui/components/Button/Button';
import type { IconKindValue } from '../../../shared/defguard-ui/components/Icon/icon-types';
import { SizedBox } from '../../../shared/defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../../shared/defguard-ui/types';
import enrollDefaultImage from './assets/enroll-default.png';
import enrollHoverImage from './assets/enroll-hover.png';
import passwordDefaultImage from './assets/password-default.png';
import passwordHoverImage from './assets/password-hover.png';

export const HomeChoice = () => {
  return (
    <div id="home-choice">
      <Card
        img="enroll"
        title={m.start_multi_enrollment_title()}
        subtitle={m.start_multi_enrollment_subtitle()}
        buttonText={m.start_multi_enrollment_button()}
        buttonIcon="arrow-big"
        link="/download"
        onClick={() => {}}
      />
      <Card
        img="password"
        title={m.start_multi_password_title()}
        subtitle={m.start_multi_password_subtitle()}
        buttonText={m.start_multi_password_button()}
        buttonIcon="lock-open"
        link="/password"
        onClick={() => {}}
      />
    </div>
  );
};

type CardProps = {
  img: 'enroll' | 'password';
  link: '/password' | '/download';
  buttonIcon: IconKindValue;
  buttonText: string;
  subtitle: string;
  title: string;
  card?: boolean;
  onClick: () => void;
};

const Card = ({
  buttonIcon,
  buttonText,
  subtitle,
  title,
  onClick,
  link,
  img,
}: CardProps) => {
  return (
    <div className={clsx('choice')}>
      <div className="image">
        {img === 'enroll' && (
          <>
            <img src={enrollDefaultImage} />
            <img src={enrollHoverImage} />
          </>
        )}
        {img === 'password' && (
          <>
            <img src={passwordDefaultImage} />
            <img src={passwordHoverImage} />
          </>
        )}
      </div>
      <SizedBox height={37} />
      <p className="title">{title}</p>
      <SizedBox height={ThemeSpacing.Md} />
      <p className="subtitle">{subtitle}</p>
      <SizedBox height={ThemeSpacing.Xl2} />
      <Link to={link}>
        <Button
          size="primary"
          variant="primary"
          text={buttonText}
          iconLeft={buttonIcon}
          onClick={onClick}
        />
      </Link>
    </div>
  );
};
