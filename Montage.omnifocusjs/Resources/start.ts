(() => {
  var action = new PlugIn.Action(async () => {
    try {
      new Alert("hello, world!", "YO").show();
    } catch (err) {
      console.error(err);
      throw err;
    }
  });

  return action;
})();
