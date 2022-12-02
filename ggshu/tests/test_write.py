import os
from ggshu import aes, geom_arrow, geom_kde, ggmap, geom_metabolite


def test_writing_does_not_raise(df_cond):
    file_name = "some_tmp_data"
    (
        ggmap(
            df_cond,
            aes(reaction="r", color="flux", size="flux", y="kcat", condition="cond"),
        )
        + geom_arrow()
        + geom_metabolite(aes=aes(color="conc", metabolite="m"))
        + geom_kde(aes=aes(y="km"), mets=True)
    ).to_json(file_name)
    assert os.path.exists(file_name + ".metabolism.json")
    os.remove(file_name + ".metabolism.json")
