local R, C, r = {}, {}, 0
for i = 0, 8 do
    for j = 0, 8 do
        for k = 0, 8 do
            C[r], r = { 9 * i + j, math.floor(i/3)*27 + math.floor(j/3)*9 + k + 81,
                        9 * i + k + 162, 9 * j + k + 243 }, r + 1
        end
    end
end