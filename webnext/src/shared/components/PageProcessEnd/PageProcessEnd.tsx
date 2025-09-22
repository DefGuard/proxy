import './style.scss';

import { Link } from '@tanstack/react-router';
import { Button } from '../../defguard-ui/components/Button/Button';
import { Icon } from '../../defguard-ui/components/Icon';
import { SizedBox } from '../../defguard-ui/components/SizedBox/SizedBox';
import { AppText } from '../../defguard-ui/components/Text/Text';
import { TextStyle, ThemeSpacing } from '../../defguard-ui/types';
import { Page } from '../Page/Page';

type Props = {
  title: string;
  subtitle: string;
  linkText: string;
  link: '/';
};

export const PageProcessEnd = ({ link, linkText, subtitle, title }: Props) => {
  return (
    <Page className="page-process-end">
      <SizedBox height={ThemeSpacing.Xl9} />
      <Icon icon="check-circle" size={20} />
      <SizedBox height={ThemeSpacing.Xl} />
      <AppText as="h1" font={TextStyle.TTitleH3}>
        {title}
      </AppText>
      <AppText as="p" font={TextStyle.TBodyPrimary400}>
        {subtitle}
      </AppText>
      <SizedBox height={ThemeSpacing.Xl3} />
      <Link to={link} replace>
        <Button text={linkText} />
      </Link>
    </Page>
  );
};
