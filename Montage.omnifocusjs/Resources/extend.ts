(() => {
  var action = new PlugIn.Action(async () => {
    try {
      let focusForm = new Form();
      focusForm.addField(new Form.Field.String("minutes", "Minutes", "5"));

      await focusForm.show("Extend session", "Extend");
      let values = focusForm.values as {
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
          "mutation ExtendByMutation($duration: String!) { extendBy(duration: $duration) { projectedEndTime } }",
        variables: {
          duration: `PT${values.minutes}M`,
        },
      });
      req.headers = { "Content-Type": "application/json" };

      let resp = await req.fetch().catch((err) => {
        console.error("Problem fetching tasks:", err);
        let alert = new Alert("Problem extending session in Montage", err);
        alert.show();
        throw err;
      });

      if (resp.bodyString === null) {
        throw "body string was null. Did the request succeed?";
      }

      console.log(`${resp.bodyString}`);

      let data = JSON.parse(resp.bodyString).data.extendBy as {
        projectedEndTime: string;
      };

      console.log(JSON.stringify(data));
      let endTime = new Date(data.projectedEndTime);

      new Alert(
        "Extended session",
        `Extended sesion until ${endTime.getHours()}:${endTime.getMinutes()}`,
      ).show();
    } catch (err) {
      console.error(err);
    }
  });

  return action;
})();
