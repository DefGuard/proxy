import './style.scss';

import { Card } from '../../../../../../shared/components/layout/Card/Card';
import { useI18nContext } from '../../../../../../i18n/i18n-react';
import { MessageBox } from '../../../../../../shared/components/layout/MessageBox/MessageBox';
import { Button } from '../../../../../../shared/components/layout/Button/Button';

export const QuickGuideCard = () => {
  const { LL } = useI18nContext();

  const cardLL = LL;

  return (
    <Card id="device-setup-guide">
      <h3></h3>
      <MessageBox message={} />
      <label></label>
      <p></p>
      <Button />
      <label></label>
      <p></p>
      <label></label>
      <p></p>
      <label></label>
      <p></p>
    </Card>
  );
};
