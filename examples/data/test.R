library(tidyverse)

df <- read_csv("mtcars.csv")

formula <- mpg ~ wt + hp + cyl + wt:hp + poly(disp, 4) - 1

colsn <- c("wt","hp","cyl","poly_disp_1","poly_disp_2","poly_disp_3","poly_disp_4","wt_x_hp")
sels <- c("wt","hp","cyl","wt_x_hp","poly_disp_1","poly_disp_2","poly_disp_3","poly_disp_4")

mm <- model.matrix(formula, df)

# Rename in the right order
colnames(mm) <- colsn

# Turn into tibble and select columns in your desired order
mm_tbl <- as_tibble(mm) |>
  select(all_of(sels))

mm_tbl |> write_csv("mtcars_poly_4.csv")


polym
norm2


qr
