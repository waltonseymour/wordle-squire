import React, { useEffect, useRef, useState } from "react";
import "./App.css";

import { Button, Text, Heading } from "evergreen-ui";

type GuessState = "Correct" | "WrongPlace" | "Missing";

function nextState(state: GuessState): GuessState {
  switch (state) {
    case "Correct":
      return "Missing";
    case "WrongPlace":
      return "Correct";
    case "Missing":
      return "WrongPlace";
  }
}

const Word: React.FC<{
  guess?: string;
  guessState: Array<GuessState>;
  onGuessStateChange: (state: Array<GuessState>) => void;
}> = (props) => {
  const onClick = (index: number) => () => {
    const newState = props.guessState.slice();
    newState[index] = nextState(props.guessState[index]);
    props.onGuessStateChange(newState);
  };

  return (
    <div
      style={{
        display: "flex",
        gap: "5px",
      }}
    >
      <Tile
        result={props.guessState[0]}
        letter={props.guess?.[0]}
        onClick={onClick(0)}
      />
      <Tile
        result={props.guessState[1]}
        letter={props.guess?.[1]}
        onClick={onClick(1)}
      />
      <Tile
        result={props.guessState[2]}
        letter={props.guess?.[2]}
        onClick={onClick(2)}
      />
      <Tile
        result={props.guessState[3]}
        letter={props.guess?.[3]}
        onClick={onClick(3)}
      />
      <Tile
        result={props.guessState[4]}
        letter={props.guess?.[4]}
        onClick={onClick(4)}
      />
    </div>
  );
};

function getColor(state: GuessState) {
  switch (state) {
    case "Missing":
      return "#787c7e";
    case "WrongPlace":
      return "#c9b458";
    case "Correct":
      return "#6aaa64";
  }
}

const Tile: React.FC<{
  letter?: string;
  result?: GuessState;
  onClick: () => void;
}> = (props) => {
  const result = props.result || "Missing";

  return (
    <div
      onClick={props.onClick}
      style={{
        display: "inline-flex",
        justifyContent: "center",
        userSelect: "none",
        alignItems: "center",
        width: "62px",
        height: "62px",
        background: getColor(result),
        fontSize: "2rem",
        lineHeight: "2rem",
        color: "white",
        textTransform: "uppercase",
        fontWeight: "bold",
      }}
    >
      {props.letter}
    </div>
  );
};

const App: React.FC = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [guess, setGuess] = useState("");
  const [guessState, setGuessState] = useState<Array<GuessState>>(
    Array(25).fill("Missing")
  );

  const guessRef = useRef("");

  const [solutions, setSolutions] = useState([]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Backspace") {
        guessRef.current = guessRef.current.slice(0, -1);
        setGuess(guessRef.current);
      } else if (
        guessRef.current.length < 25 &&
        e.key.length === 1 &&
        e.key !== " "
      ) {
        guessRef.current += e.key;
        setGuess(guessRef.current);
      }
    };

    document.addEventListener("keydown", handler);

    return () => {
      document.removeEventListener("keypress", handler);
    };
  }, []);

  return (
    <>
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          margin: "12px",
          position: "absolute",
        }}
      >
        <Heading style={{ fontSize: "24px" }}>Wordle Squire</Heading>

        <Text style={{ marginTop: "10px", maxWidth: "600px" }}>
          Type your wordle guess and click the tiles to set the result. <br />
          Wordle Squire will generate every possible word that satifies your
          clues, sorted by word frequency in google scholar.
        </Text>
      </div>

      <Text
        size="small"
        style={{
          left: "8px",
          bottom: "8px",
          position: "fixed",
        }}
      >
        made by{" "}
        <a href="https://github.com/waltonseymour/wordle-squire">
          @waltonseymour
        </a>
      </Text>
      <div style={{ display: "flex", flexDirection: "row" }}>
        <div
          className="guesses"
          style={{
            display: "flex",
            flexDirection: "column",
            width: "100%",
            maxWidth: "500px",
            height: "100vh",
            margin: "0 auto",
            justifyContent: "center",
            alignItems: "center",
            gap: "10px",
          }}
        >
          {Array(5)
            .fill(undefined)
            .map((_, i) => {
              const start = i * 5;
              const end = (i + 1) * 5;
              return (
                <Word
                  key={i}
                  guess={guess.slice(start, end)}
                  guessState={guessState.slice(start, end)}
                  onGuessStateChange={(newState) => {
                    let newGuessState = [...guessState];
                    newState.forEach((x, i) => (newGuessState[start + i] = x));
                    setGuessState(newGuessState);
                  }}
                />
              );
            })}

          <Button
            isLoading={isLoading}
            appearance="primary"
            fontSize="18px"
            style={{ width: "330px" }}
            disabled={guess.length === 0 || guess.length % 5 !== 0}
            onClick={async () => {
              setIsLoading(true);
              const resp = await fetch(
                "https://server-vistk7eaba-uk.a.run.app/words",
                {
                  method: "POST",
                  headers: { "Content-Type": "application/json" },
                  body: JSON.stringify(
                    Array(guess.length / 5)
                      .fill(undefined)
                      .map((x, i) => {
                        const start = i * 5;
                        const end = (i + 1) * 5;
                        return {
                          guess: guess.slice(start, end),
                          result: guessState.slice(start, end),
                        };
                      })
                  ),
                }
              );
              const parsed = await resp.json();

              setSolutions(parsed);
              setIsLoading(false);
            }}
          >
            Find Possible Solutions
          </Button>
        </div>

        <div className="possible-solutions">
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              width: "250px",
            }}
          >
            {solutions.length > 0 && (
              <Heading textAlign="center">
                {solutions.length} {solutions.length === 1 ? "word" : "words"}{" "}
                found
              </Heading>
            )}

            {solutions.map((x) => {
              return (
                <div
                  style={{
                    background: "#6aaa64",
                    justifyContent: "center",
                    margin: "3px",
                    padding: "2px",
                    display: "flex",
                    userSelect: "none",
                    alignItems: "center",
                    fontSize: "1.4rem",
                    color: "white",
                    borderRadius: "5px",
                    textTransform: "uppercase",
                    fontWeight: "bold",
                  }}
                  key={x}
                >
                  {x}
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </>
  );
};

export default App;
