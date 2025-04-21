import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button, Card, H3, H2, H1 } from "@blueprintjs/core";
import Container from 'react-bootstrap/Container';
import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';


function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [ads, setAds] = useState(null);
  const [empty, setEmpty] = useState(false);
  const [loading, setLoading] = useState(false);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }
  async function getAds(): Array<Array<string>> {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    const ads = await invoke("list_agents_and_daemons");
    setAds(ads);
    return ads
  }

  if (ads === null) {
    setLoading(true);
    setTimeout(()=>{
      getAds().then(ads => {
        setLoading(false);
        setEmpty(ads.length === 0);
      });
    }, 10);
  }

  return (
    <Container>
      <Row>
        <Col>
          <Card>
            <Container>
              <Row>
                <Col>
                  <H3>CDB-ADM</H3>
                </Col>
                <Col>
                  <H2>Manage</H2>
                </Col>
                <Col>
                  <H1>Agents and Daemons</H1>
                </Col>
                <Col>
                </Col>
                <Col>
                </Col>
              </Row>
            </Container>
            <p>Embark on an epic journey across uncharted lands. This card outlines your mission.</p>
            <Button intent="primary">Start Journey</Button>
          </Card>
        </Col>
      </Row>
      <Row>
        <Card>
          <Container>
            <Row>
              <Col md="3">SERVICE</Col>
              <Col md="1">PID</Col>
              <Col md="2">DOMAIN</Col>
              <Col md="2">STATUS</Col>
              <Col md="4">PATH</Col>
            </Row>
          </Container>
        </Card>
      </Row>

      {Array.from(ads || []).map(([service, pid, domain, status, path]) => (
        <Row>
          <Card>
            <Container>
              <Row>
                <Col md="3">{service}</Col>
                <Col md="1">{pid}</Col>
                <Col md="2">{domain}</Col>
                <Col md="2">{status}</Col>
                <Col md="4">{path}</Col>
              </Row>
            </Container>
          </Card>
        </Row>
      ))}
    </Container>
  );
}

export default App;
