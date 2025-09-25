import { type PropsWithChildren, useState } from 'react';
import { Page } from '../../shared/components/Page/Page';
import './style.scss';
import { Avatar } from '../../shared/defguard-ui/components/Avatar/Avatar';
import { Badge } from '../../shared/defguard-ui/components/Badge/Badge';
import { Button } from '../../shared/defguard-ui/components/Button/Button';
import { Checkbox } from '../../shared/defguard-ui/components/Checkbox/Checkbox';
import { CounterLabel } from '../../shared/defguard-ui/components/CounterLabel/CounterLabel';
import { Divider } from '../../shared/defguard-ui/components/Divider/Divider';
import { EmptyState } from '../../shared/defguard-ui/components/EmptyState/EmptyState';
import { IconButton } from '../../shared/defguard-ui/components/IconButton/IconButton';
import { Modal } from '../../shared/defguard-ui/components/Modal/Modal';
import { ModalControls } from '../../shared/defguard-ui/components/ModalControls/ModalControls';
import { Radio } from '../../shared/defguard-ui/components/Radio/Radio';

export const TestPage = () => {
  return (
    <Page id="test-page">
      <TestRow>
        <h1>Test ground</h1>
      </TestRow>
      <TestRow>
        <Button text="Test button primary" />
        <Button text="Test button primary" loading />
        <Button text="Test button primary" disabled />
      </TestRow>
      <TestRow>
        <Button text="Test button" variant="secondary" />
        <Button text="Test button" variant="secondary" loading />
        <Button text="Test button" variant="secondary" disabled />
      </TestRow>
      <TestRow>
        <Button text="Test button" variant="outlined" />
        <Button text="Test button" variant="outlined" loading />
        <Button text="Test button" variant="outlined" disabled />
      </TestRow>
      <TestRow>
        <Button text="Test button" variant="critical" />
        <Button text="Test button" variant="critical" loading />
        <Button text="Test button" variant="critical" disabled />
      </TestRow>
      <TestRow>
        <TestButtonTransition />
      </TestRow>
      <TestRow>
        <IconButton icon="plus" />
        <IconButton icon="plus" />
        <IconButton icon="plus" disabled />
      </TestRow>
      <TestRow>
        <Divider orientation="horizontal" />
      </TestRow>
      <TestRow>
        <Divider orientation="vertical" />
      </TestRow>
      <TestRow>
        <Divider text="Divider test" orientation="horizontal" />
      </TestRow>
      <TestRow>
        <Checkbox active={false} />
        <Checkbox active={false} />
        <Checkbox active={false} disabled />
        <Checkbox active={true} />
        <Checkbox active={true} disabled />
        <Checkbox error="some error" />
      </TestRow>
      <TestRow>
        <Checkbox text="Test checkbox" />
        <Checkbox active text="Test checkbox" />
        <Checkbox text="Test checkbox" disabled />
        <Checkbox text="test checkbox" error="some error" />
      </TestRow>
      <TestRow>
        <Radio />
        <Radio active />
        <Radio disabled />
        <Radio active disabled />
      </TestRow>
      <TestRow>
        <Radio text="Radio" />
        <Radio text="Radio" active />
        <Radio text="Radio" disabled />
        <Radio text="Radio" active disabled />
      </TestRow>
      <TestRow>
        <Avatar size="small" />
        <Avatar />
        <Avatar size="big" />
      </TestRow>
      <TestRow>
        <Badge text="Neutral" variant="neutral" />
        <Badge text="Success" variant="success" />
        <Badge text="Critical" variant="critical" />
        <Badge text="Warning" variant="warning" />
      </TestRow>
      <TestRow>
        <Badge text="Neutral" variant="neutral" background />
        <Badge text="Success" variant="success" background />
        <Badge text="Critical" variant="critical" background />
        <Badge text="Warning" variant="warning" background />
      </TestRow>
      <TestRow>
        <EmptyState
          icon="arrow-big"
          title="No Yubikey stations registered"
          subtitle="Add your first provision station by clicking the button below."
          primaryAction={{
            text: 'Button',
            onClick: () => {},
          }}
          secondaryAction={() => {}}
          secondaryActionText="Secondary action"
        />
      </TestRow>
      <TestRow>
        <CounterLabel value={0} variant="neutral" />
        <CounterLabel value={100} variant="action" />
        <CounterLabel value={1000} variant="critical" />
        <CounterLabel value={10000} variant="warning" />
        <CounterLabel value={100000} variant="default" />
      </TestRow>
      <TestRow>
        <TestModalButton />
      </TestRow>
    </Page>
  );
};

const TestRow = ({ children }: PropsWithChildren) => {
  return <div className="test-row">{children}</div>;
};

const TestButtonTransition = () => {
  const [loading, setLoading] = useState(false);

  return (
    <Button
      text="test loader"
      loading={loading}
      onClick={() => {
        setLoading((s) => !s);
        setTimeout(() => {
          setLoading(false);
        }, 1000);
      }}
    />
  );
};

const TestModalButton = () => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      <Button
        text="Open test modal"
        variant="primary"
        size="big"
        onClick={() => {
          setIsOpen(true);
        }}
      />
      <Modal
        title="Test modal"
        isOpen={isOpen}
        onClose={() => {
          setIsOpen(false);
        }}
      >
        <p>Test modal content text.</p>
        <ModalControls />
      </Modal>
    </>
  );
};
