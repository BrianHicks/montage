(() => {
  var action = new PlugIn.Action(async (selection: Selection) => {
    try {
      let suggestedDescription: string | null = null;
      let suggestedMinutes: string = "25";

      if (selection.tasks[0]) {
        let task: Task = selection.tasks[0];

        if (task.estimatedMinutes) {
          suggestedMinutes = task.estimatedMinutes.toString();
        }
        suggestedDescription = task.name;
      } else if (selection.tags) {
        suggestedDescription = selection.tags[0].name;
      } else if (selection.projects[0]) {
        let project = selection.projects[0];

        suggestedDescription = project.name;
      }

      let focusForm = new Form();
      focusForm.addField(
        new Form.Field.String(
          "description",
          "Description",
          suggestedDescription
        )
      );
      focusForm.addField(
        new Form.Field.String("minutes", "Minutes", suggestedMinutes)
      );

      await focusForm.show("Start a session", "Start");
      let values = focusForm.values as {
        description: string;
        minutes: string;
      };

      console.log(values);

      /// Step 2: start the session!
      let req = URL.FetchRequest.fromString("http://localhost:4774/graphql");
      if (!req?.url?.host) {
        throw "could not parse the URL for the Montage API";
      }

      req.method = "POST";
      req.bodyString = JSON.stringify({
        query:
          "mutation StartMutation($description: String!, $kind: Kind!, $duration: Duration) { start(description: $description, kind: $kind, duration: $duration) { duration projectedEndTime } }",
        variables: {
          description: values.description,
          kind: "TASK",
          duration: `PT${values.minutes}M`,
        },
      });
      req.headers = { "Content-Type": "application/json" };

      let resp = await req.fetch().catch((err) => {
        console.error("Problem fetching tasks:", err);
        let alert = new Alert("Problem starting session in Montage", err);
        alert.show();
        throw err;
      });

      if (resp.bodyString === null) {
        throw "body string was null. Did the request succeed?";
      }

      let data = JSON.parse(resp.bodyString).data.start as {
        duration: string;
        projectedEndTime: string;
      };

      console.log(JSON.stringify(data));
      let minutes = parseInt(data.duration.match(/PT(\d+)S/)![1], 10);

      let endTime = new Date(data.projectedEndTime);

      new Alert(
        "Started session",
        `Started task for ${Math.round(
          minutes / 60
        )} minutes, until ${endTime.getHours()}:${endTime.getMinutes()}`
      ).show();
    } catch (err) {
      console.error(err);
    }
  });

  action.validate = function (selection: Selection) {
    return (
      selection.tasks.length === 1 ||
      selection.tags.length === 1 ||
      selection.projects.length === 1
    );
  };

  return action;
})();
