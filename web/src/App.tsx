import React, { useEffect, useRef, useState } from "react";
import "./App.css";

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
  const [guess, setGuess] = useState("");
  const [guessState, setGuessState] = useState<Array<GuessState>>([
    "Missing",
    "Missing",
    "Missing",
    "Missing",
    "Missing",
  ]);

  const guessRef = useRef("");

  const [solutions, setSolutions] = useState([]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      console.log(e);
      if (e.key === "Backspace") {
        guessRef.current = guessRef.current.slice(0, -1);
        setGuess(guessRef.current);
      } else if (guessRef.current.length < 5 && e.key.length === 1) {
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
    <div style={{ display: "flex", flexDirection: "row" }}>
      <div
        className="guesses"
        style={{
          display: "flex",
          width: "100%",
          maxWidth: "500px",
          margin: "0 auto",
          justifyContent: "center",
        }}
      >
        <Word
          guess={guess}
          guessState={guessState}
          onGuessStateChange={setGuessState}
        />
      </div>
      <div className="possible-solutions">
        <button
          onClick={async () => {
            const resp = await fetch(
              "https://server-vistk7eaba-uk.a.run.app/words",
              {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify([{ guess, result: guessState }]),
              }
            );
            const parsed = await resp.json();

            setSolutions(parsed);
          }}
        >
          Generate Possible Solutions
        </button>

        {solutions.map((x) => {
          return <div key={x}>{x}</div>;
        })}
      </div>
    </div>
  );
};

export default App;
