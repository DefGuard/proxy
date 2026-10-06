import './style.scss';

import { Link } from '@tanstack/react-router';
import { AppText } from '../../defguard-ui/components/AppText/AppText';
import { Button } from '../../defguard-ui/components/Button/Button';
import { Icon } from '../../defguard-ui/components/Icon';
import type { IconKindValue } from '../../defguard-ui/components/Icon/icon-types';
import { SizedBox } from '../../defguard-ui/components/SizedBox/SizedBox';
import { TextStyle, ThemeSpacing } from '../../defguard-ui/types';
import { isPresent } from '../../defguard-ui/utils/isPresent';
import { Page } from '../Page/Page';

type Props = {
  title: string;
  subtitle: string;
  linkText?: string;
  link?: string;
  icon?: IconKindValue;
  imageSrc?: string;
};

export const PageInfo = ({
  link,
  linkText,
  subtitle,
  title,
  icon = 'check-circle',
  imageSrc,
}: Props) => {
  return (
    <Page className="page-info">
      <div className="content">
        {imageSrc ? <img src={imageSrc} /> : <Icon icon={icon} size={32} />}
        <SizedBox height={ThemeSpacing.Xl} />
        <AppText as="h1" font={TextStyle.TTitleH3}>
          {title}
        </AppText>
        <SizedBox height={ThemeSpacing.Xs} />
        <AppText as="p" font={TextStyle.TBodyPrimary400}>
          {subtitle}
        </AppText>
        {isPresent(linkText) && isPresent(link) && (
          <>
            <SizedBox height={ThemeSpacing.Xl3} />
            {link.includes('://') ? (
              <a href={link}>
                <Button text={linkText} />
              </a>
            ) : (
              <Link to={link} replace>
                <Button text={linkText} />
              </Link>
            )}
          </>
        )}
      </div>
    </Page>
  );
};
